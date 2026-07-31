//! Uniform-grid spatial index (broadphase) — Phase 2 of the WASM/ECS
//! migration plan (`.tools/docs/engineering/WASM_ECS_MIGRATION_PLAN.md`).
//!
//! Ports the design already proven in the JS game's own grid
//! (`index.html` ~line 5178: `spatialGrid`/`gridKey`/`rebuildSpatialGrid`/
//! `forEachEnemyNear`) rather than inventing a new one. At this game's
//! scale (≤1600 entities, a bounded world a few thousand units per side) a
//! uniform grid is the right structure — no quadtree/BVH here.
//!
//! Two deliberate differences from the JS version:
//! - JS bucketed into a `Map` keyed by an offset-encoded integer — a
//!   workaround for string-concatenation keys being slow. Rust needs no
//!   such workaround: buckets are indexed by a flat `Vec`, plain array
//!   indexing (`cell_y * grid_w + cell_x`), which is both simpler and
//!   faster than a hash map lookup.
//! - World bounds are a constructor parameter, not a hardcoded constant.
//!   This crate is otherwise game-agnostic; baking in one game's
//!   4600×4600 world here would be the wrong layer for that number to
//!   live in. Only the cell size (`CELL_SIZE`) is a crate-level constant,
//!   matching the JS `GRID_CELL` value, since bucket granularity is a
//!   property of the algorithm, not of any particular game's world.
//!
//! The single most load-bearing property, carried over unchanged from the
//! JS version: bucket storage is **reused**, never reallocated, after the
//! first `rebuild()` populates it. Every subsequent `rebuild()` only
//! `.clear()`s existing `Vec`s and refills them — the bounded world means
//! the total cell count (and therefore total bucket count) is fixed for
//! the lifetime of the grid, so there is nothing left to allocate once
//! warm. `rebuild()` is structured so that property is obvious from
//! reading it: there is no `self.buckets[i] = Vec::new()` anywhere in it.

use crate::Entity;

/// Cell size in world units. Matches the JS game's `GRID_CELL` constant.
/// Kept as a named constant (not inlined) so it's easy to retune later
/// without hunting for a magic number.
pub const CELL_SIZE: f32 = 160.0;

/// One bucketed entity: its id plus the position it was inserted at. The
/// position is kept alongside the id (not re-fetched from the ECS) so
/// `query_near`'s precise-distance filter can run directly off grid data.
type Entry = (Entity, f32, f32);

/// A uniform spatial grid over a bounded 2D world, used as a broadphase:
/// "which entities are roughly near this point" before a precise (and
/// usually more expensive) check.
pub struct SpatialGrid {
    cell_size: f32,
    grid_w: i32,
    grid_h: i32,
    buckets: Vec<Vec<Entry>>,
}

impl SpatialGrid {
    /// Build a grid covering a `[0, world_w] x [0, world_h]` world with the
    /// given cell size. Bucket count is fixed at construction time (`ceil
    /// (world_w/cell_size) * ceil(world_h/cell_size)`) and never changes —
    /// that's what makes the "never reallocate, only clear" property of
    /// `rebuild` possible.
    pub fn new(cell_size: f32, world_w: f32, world_h: f32) -> Self {
        let grid_w = ((world_w / cell_size).ceil() as i32).max(1);
        let grid_h = ((world_h / cell_size).ceil() as i32).max(1);
        let cell_count = (grid_w as usize) * (grid_h as usize);
        Self {
            cell_size,
            grid_w,
            grid_h,
            buckets: (0..cell_count).map(|_| Vec::new()).collect(),
        }
    }

    /// Cell coordinates for a world position, clamped into grid bounds.
    /// Entities outside the nominal `[0, world_w] x [0, world_h]` world
    /// (which does happen in practice — projectiles/knockback can push a
    /// few units past an edge) land in the nearest edge cell instead of
    /// panicking or being silently dropped.
    #[inline]
    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        (cx.clamp(0, self.grid_w - 1), cy.clamp(0, self.grid_h - 1))
    }

    #[inline]
    fn bucket_index(&self, cx: i32, cy: i32) -> usize {
        (cy as usize) * (self.grid_w as usize) + (cx as usize)
    }

    /// Repopulate the grid from scratch for one frame. Takes any iterator
    /// of `(Entity, x, y)` — the caller (in this crate, `wasm_api.rs`)
    /// decides where those positions come from; this module has no ECS
    /// dependency beyond the `Entity` type, which keeps it plainly
    /// unit-testable.
    ///
    /// Bucket `Vec`s are `.clear()`'d, never replaced — no allocation
    /// happens here once the grid has been warmed up once (see module
    /// docs).
    pub fn rebuild<I: IntoIterator<Item = Entry>>(&mut self, entities: I) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        for (entity, x, y) in entities {
            let (cx, cy) = self.cell_coords(x, y);
            let idx = self.bucket_index(cx, cy);
            self.buckets[idx].push((entity, x, y));
        }
    }

    /// Cell range `[min, max]` (inclusive, already clamped to grid bounds)
    /// that a query circle of `radius` centered at `(x, y)` overlaps, on
    /// one axis. Shared by `query_near`/`query_near_count`'s two-step scan
    /// (overlapping cells, then precise-distance filter) — same shape as
    /// the JS version's `forEachEnemyNear`.
    ///
    /// The center cell is clamped the same way `cell_coords` clamps an
    /// entity position (not independently-clamped raw endpoints) — that
    /// keeps this consistent with insertion: a query centered on the same
    /// out-of-bounds point an out-of-bounds entity was inserted at must
    /// land on the same clamped edge cell that entity was bucketed into,
    /// or it could never find it. Always returns a valid (non-empty)
    /// range since the clamped center cell is always in bounds.
    #[inline]
    fn axis_cell_range(&self, center: f32, radius: f32, axis_len: i32) -> (i32, i32) {
        let center_cell = (center / self.cell_size).floor() as i32;
        let center_cell = center_cell.clamp(0, axis_len - 1);
        let span = (radius / self.cell_size).ceil() as i32;
        let min_c = (center_cell - span).max(0);
        let max_c = (center_cell + span).min(axis_len - 1);
        (min_c, max_c)
    }

    /// Scan every cell the `(x, y, radius)` circle overlaps, calling `f`
    /// for each bucketed entity that is within `radius` of the center by
    /// actual (not just cell-bucket) distance. Shared implementation for
    /// `query_near` (collects) and `query_near_count` (counts only, no
    /// allocation).
    fn for_each_near<F: FnMut(Entity, f32, f32)>(&self, x: f32, y: f32, radius: f32, mut f: F) {
        let (min_cx, max_cx) = self.axis_cell_range(x, radius, self.grid_w);
        let (min_cy, max_cy) = self.axis_cell_range(y, radius, self.grid_h);
        let r2 = radius * radius;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                let idx = self.bucket_index(cx, cy);
                for &(entity, ex, ey) in &self.buckets[idx] {
                    let dx = ex - x;
                    let dy = ey - y;
                    if dx * dx + dy * dy <= r2 {
                        f(entity, ex, ey);
                    }
                }
            }
        }
    }

    /// All entities within `radius` of `(x, y)`.
    pub fn query_near(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let mut out = Vec::new();
        self.for_each_near(x, y, radius, |e, _, _| out.push(e));
        out
    }

    /// Count of entities within `radius` of `(x, y)`, without allocating a
    /// result vec — matches the `query_count`-style "count-only avoids
    /// allocation" convention already used in `wasm_api.rs`.
    pub fn query_near_count(&self, x: f32, y: f32, radius: f32) -> usize {
        let mut count = 0usize;
        self.for_each_near(x, y, radius, |_, _, _| count += 1);
        count
    }

    /// Total number of buckets (fixed at construction, exposed mainly for
    /// tests/diagnostics).
    pub fn cell_count(&self) -> usize {
        self.buckets.len()
    }
}
