//! WASM/JS API — компилируется только при feature = "wasm"
//!
//! Exposes a flat, allocation-minimal API for JavaScript.
//! All data crossing the JS↔WASM boundary is passed as typed arrays
//! (Float32Array, Uint32Array) to avoid per-call allocations.

#![cfg(feature = "wasm")]

use std::collections::HashMap;

use js_sys::{Float32Array, Uint32Array};
use wasm_bindgen::prelude::*;

use crate::{ComponentBuilder, ComponentInstance, ComponentValue, FieldType, SpatialGrid, World, CELL_SIZE};

/// The main JS-facing ECS world handle.
///
/// ```js
/// import init, { JsWorld } from './pkg/ecs_core.js';
/// await init();
/// const world = new JsWorld();
/// ```
#[wasm_bindgen]
pub struct JsWorld {
    inner: World,
    /// One spatial broadphase grid per position-shaped component id. Keyed
    /// by component id (not a single fixed grid) because the JS game's own
    /// grid already serves more than one query need — enemies today,
    /// bullets/orbs later — see `spatial.rs` module docs.
    spatial_grids: HashMap<u32, SpatialGrid>,
}

#[wasm_bindgen]
impl JsWorld {
    /// Create a new ECS world
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsWorld {
        JsWorld {
            inner: World::new(),
            spatial_grids: HashMap::new(),
        }
    }

    // ── Component registration ────────────────────────────────────────────

    /// Register a 2×f32 component (e.g. Position, Velocity)
    /// Returns the component ID.
    ///
    /// ```js
    /// const posId = world.register_f32x2('Position');
    /// ```
    pub fn register_f32x2(&mut self, name: &str) -> u32 {
        self.inner.register_component(
            ComponentBuilder::new(name)
                .field("x", FieldType::F32)
                .field("y", FieldType::F32)
                .build(),
        )
    }

    /// Register a 3×f32 component (e.g. Position3D, Color)
    pub fn register_f32x3(&mut self, name: &str) -> u32 {
        self.inner.register_component(
            ComponentBuilder::new(name)
                .field("x", FieldType::F32)
                .field("y", FieldType::F32)
                .field("z", FieldType::F32)
                .build(),
        )
    }

    /// Register a 1×f32 component (e.g. Health, Speed)
    pub fn register_f32(&mut self, name: &str) -> u32 {
        self.inner.register_component(
            ComponentBuilder::new(name)
                .field("value", FieldType::F32)
                .build(),
        )
    }

    /// Register a 1×u32 component (e.g. Team, State enum)
    pub fn register_u32(&mut self, name: &str) -> u32 {
        self.inner.register_component(
            ComponentBuilder::new(name)
                .field("value", FieldType::U32)
                .build(),
        )
    }

    /// Register a 1×bool component (e.g. Active, Visible)
    pub fn register_bool(&mut self, name: &str) -> u32 {
        self.inner.register_component(
            ComponentBuilder::new(name)
                .field("value", FieldType::Bool)
                .build(),
        )
    }

    /// Look up a component ID by its registered name.
    /// Returns `u32::MAX` if not found.
    ///
    /// ```js
    /// const posId = world.get_component_id('Position');
    /// ```
    pub fn get_component_id(&self, name: &str) -> u32 {
        self.inner
            .component_registry
            .get_by_name(name)
            .map(|s| s.id)
            .unwrap_or(u32::MAX)
    }

    /// Get component schema as a JSON string.
    /// Returns `"null"` if the component ID is not registered.
    ///
    /// ```js
    /// const schema = JSON.parse(world.get_component_schema(posId));
    /// // { id: 0, name: "Position", size: 8, fields: [...] }
    /// ```
    pub fn get_component_schema(&self, component_id: u32) -> String {
        self.inner
            .component_registry
            .get(component_id)
            .map(|s| s.describe())
            .unwrap_or_else(|| "null".to_string())
    }

    // ── Entity spawn ─────────────────────────────────────────────────────

    /// Spawn an empty entity (no components). Returns the entity ID.
    /// Call `entity_generation(id)` to get the generation for `despawn`.
    ///
    /// ```js
    /// const id = world.spawn_empty_entity();
    /// const gen = world.entity_generation(id);
    /// world.add_component_f32x2(id, gen, posId, 10, 20);
    /// ```
    pub fn spawn_empty_entity(&mut self) -> u32 {
        self.inner.spawn_empty().id
    }

    /// Spawn one entity with a single 2×f32 component.
    /// Returns entity ID (u32, generation encoded separately via entity_generation).
    pub fn spawn_f32x2(&mut self, component_id: u32, x: f32, y: f32) -> u32 {
        let e = self.inner.spawn(vec![ComponentInstance::new(
            component_id,
            vec![ComponentValue::F32(x), ComponentValue::F32(y)],
        )]);
        e.id
    }

    /// Spawn one entity with a single 1×f32 component.
    pub fn spawn_f32(&mut self, component_id: u32, value: f32) -> u32 {
        let e = self.inner.spawn(vec![ComponentInstance::new(
            component_id,
            vec![ComponentValue::F32(value)],
        )]);
        e.id
    }

    /// Spawn one entity with a single 1×u32 component.
    pub fn spawn_u32(&mut self, component_id: u32, value: u32) -> u32 {
        let e = self.inner.spawn(vec![ComponentInstance::new(
            component_id,
            vec![ComponentValue::U32(value)],
        )]);
        e.id
    }

    /// Spawn N entities with a single 2×f32 component (batch, very fast).
    /// Returns a Uint32Array of entity IDs.
    pub fn spawn_batch_f32x2(
        &mut self,
        component_id: u32,
        x: f32,
        y: f32,
        count: u32,
    ) -> Uint32Array {
        let entities = self.inner.spawn_batch(
            vec![ComponentInstance::new(
                component_id,
                vec![ComponentValue::F32(x), ComponentValue::F32(y)],
            )],
            count as usize,
        );
        let ids: Vec<u32> = entities.iter().map(|e| e.id).collect();
        let arr = Uint32Array::new_with_length(ids.len() as u32);
        arr.copy_from(&ids);
        arr
    }

    /// Spawn N entities with pos + vel f32x2 components.
    pub fn spawn_batch_pos_vel(
        &mut self,
        pos_id: u32,
        px: f32,
        py: f32,
        vel_id: u32,
        vx: f32,
        vy: f32,
        count: u32,
    ) -> Uint32Array {
        let entities = self.inner.spawn_batch(
            vec![
                ComponentInstance::new(
                    pos_id,
                    vec![ComponentValue::F32(px), ComponentValue::F32(py)],
                ),
                ComponentInstance::new(
                    vel_id,
                    vec![ComponentValue::F32(vx), ComponentValue::F32(vy)],
                ),
            ],
            count as usize,
        );
        let ids: Vec<u32> = entities.iter().map(|e| e.id).collect();
        let arr = Uint32Array::new_with_length(ids.len() as u32);
        arr.copy_from(&ids);
        arr
    }

    // ── Entity lifecycle ──────────────────────────────────────────────────

    /// Destroy an entity (O(1) swap-remove)
    pub fn despawn(&mut self, entity_id: u32, generation: u32) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner.despawn(entity).is_ok()
    }

    pub fn is_alive(&self, entity_id: u32, generation: u32) -> bool {
        self.inner.is_alive(crate::Entity {
            id: entity_id,
            generation,
        })
    }

    pub fn entity_count(&self) -> u32 {
        self.inner.entity_count() as u32
    }

    /// Get the current generation of an entity ID.
    ///
    /// Use this after any spawn call that returns only an ID to reconstruct
    /// the full entity handle for subsequent calls like `despawn` or `has_component`.
    ///
    /// ```js
    /// const id = world.spawn_f32x2(posId, 10, 20);
    /// const gen = world.entity_generation(id);   // usually 0 for new IDs
    /// world.add_component_f32(id, gen, healthId, 100);
    /// ```
    pub fn entity_generation(&self, entity_id: u32) -> u32 {
        self.inner.entity_manager.generation_of(entity_id)
    }

    // ── Component add / remove / query ────────────────────────────────────

    /// Add or update a 2×f32 component on an existing entity.
    /// Migrates to a new archetype if the entity doesn't already have this component.
    /// Returns false if the entity is dead.
    ///
    /// ```js
    /// world.add_component_f32x2(id, gen, velId, 1.5, 0.0);
    /// ```
    pub fn add_component_f32x2(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        x: f32,
        y: f32,
    ) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .add_component(
                entity,
                ComponentInstance::new(
                    component_id,
                    vec![ComponentValue::F32(x), ComponentValue::F32(y)],
                ),
            )
            .is_ok()
    }

    /// Add or update a 1×f32 component on an existing entity.
    pub fn add_component_f32(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        value: f32,
    ) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .add_component(
                entity,
                ComponentInstance::new(component_id, vec![ComponentValue::F32(value)]),
            )
            .is_ok()
    }

    /// Add or update a 1×u32 component on an existing entity.
    pub fn add_component_u32(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        value: u32,
    ) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .add_component(
                entity,
                ComponentInstance::new(component_id, vec![ComponentValue::U32(value)]),
            )
            .is_ok()
    }

    /// Add or update a 1×bool component on an existing entity.
    pub fn add_component_bool(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        value: bool,
    ) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .add_component(
                entity,
                ComponentInstance::new(component_id, vec![ComponentValue::Bool(value)]),
            )
            .is_ok()
    }

    /// Remove a component from an entity. Returns false if the entity is dead
    /// or does not have the component.
    ///
    /// ```js
    /// world.remove_component(id, gen, velocityId);
    /// ```
    pub fn remove_component(&mut self, entity_id: u32, generation: u32, component_id: u32) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner.remove_component(entity, component_id).is_ok()
    }

    /// Returns true if the entity has the given component.
    ///
    /// ```js
    /// if (world.has_component(id, gen, healthId)) { ... }
    /// ```
    pub fn has_component(&self, entity_id: u32, generation: u32, component_id: u32) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner.has_component(entity, component_id)
    }

    // ── Single-entity field read/write ────────────────────────────────────

    /// Read a single f32 field at `byte_offset` within a component.
    /// Returns 0.0 if the entity is dead or lacks the component.
    ///
    /// byte_offset: 0 = first f32 field, 4 = second, 8 = third, etc.
    ///
    /// ```js
    /// const health = world.get_f32(id, gen, healthId, 0);
    /// ```
    pub fn get_f32(
        &self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        byte_offset: usize,
    ) -> f32 {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .get_component_bytes(entity, component_id)
            .and_then(|bytes| {
                if byte_offset + 4 <= bytes.len() {
                    Some(f32::from_le_bytes(
                        bytes[byte_offset..byte_offset + 4].try_into().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    /// Write a single f32 field at `byte_offset` within a component in-place.
    /// No-op if the entity is dead or lacks the component.
    ///
    /// ```js
    /// world.set_f32(id, gen, healthId, 0, 80.0); // set health to 80
    /// ```
    pub fn set_f32(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        byte_offset: usize,
        value: f32,
    ) {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        if !self.inner.entity_manager.is_alive(entity) {
            return;
        }

        let loc = match self.inner.locations.get(entity.id as usize) {
            Some(&l) if l.is_valid() => l,
            _ => return,
        };
        let archetype = match self.inner.archetypes.get_mut(loc.archetype_id as usize) {
            Some(a) if a.has_component(component_id) => a,
            _ => return,
        };
        let col_idx = archetype.column_index(component_id);
        let chunk = &mut archetype.chunks[loc.chunk_index as usize];
        let bytes = chunk.columns[col_idx].row_mut(loc.row as usize);
        if byte_offset + 4 <= bytes.len() {
            bytes[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    /// Read a single u32 field at `byte_offset` within a component.
    /// Returns 0 if the entity is dead or lacks the component.
    pub fn get_u32(
        &self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        byte_offset: usize,
    ) -> u32 {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .get_component_bytes(entity, component_id)
            .and_then(|bytes| {
                if byte_offset + 4 <= bytes.len() {
                    Some(u32::from_le_bytes(
                        bytes[byte_offset..byte_offset + 4].try_into().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    /// Write a single u32 field at `byte_offset` within a component in-place.
    pub fn set_u32(
        &mut self,
        entity_id: u32,
        generation: u32,
        component_id: u32,
        byte_offset: usize,
        value: u32,
    ) {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        if !self.inner.entity_manager.is_alive(entity) {
            return;
        }

        let loc = match self.inner.locations.get(entity.id as usize) {
            Some(&l) if l.is_valid() => l,
            _ => return,
        };
        let archetype = match self.inner.archetypes.get_mut(loc.archetype_id as usize) {
            Some(a) if a.has_component(component_id) => a,
            _ => return,
        };
        let col_idx = archetype.column_index(component_id);
        let chunk = &mut archetype.chunks[loc.chunk_index as usize];
        let bytes = chunk.columns[col_idx].row_mut(loc.row as usize);
        if byte_offset + 4 <= bytes.len() {
            bytes[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    /// Read a single bool field at `byte_offset` (1 byte) within a
    /// component. Returns `false` if the entity is dead or lacks the
    /// component. Stage 2 shipped `add_component_bool` with no way to read
    /// the value back (`get_u32` requires 4 bytes; a bool field is 1) —
    /// this closes that gap the same way `get_f32`/`get_u32` already work.
    pub fn get_bool(&self, entity_id: u32, generation: u32, component_id: u32, byte_offset: usize) -> bool {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .get_component_bytes(entity, component_id)
            .and_then(|bytes| bytes.get(byte_offset).copied())
            .map(|b| b != 0)
            .unwrap_or(false)
    }

    /// Write a single bool field at `byte_offset` within a component in-place.
    pub fn set_bool(&mut self, entity_id: u32, generation: u32, component_id: u32, byte_offset: usize, value: bool) {
        let entity = crate::Entity {
            id: entity_id,
            generation,
        };
        if !self.inner.entity_manager.is_alive(entity) {
            return;
        }

        let loc = match self.inner.locations.get(entity.id as usize) {
            Some(&l) if l.is_valid() => l,
            _ => return,
        };
        let archetype = match self.inner.archetypes.get_mut(loc.archetype_id as usize) {
            Some(a) if a.has_component(component_id) => a,
            _ => return,
        };
        let col_idx = archetype.column_index(component_id);
        let chunk = &mut archetype.chunks[loc.chunk_index as usize];
        let bytes = chunk.columns[col_idx].row_mut(loc.row as usize);
        if byte_offset < bytes.len() {
            bytes[byte_offset] = if value { 1 } else { 0 };
        }
    }

    // ── Query (entity identity by component set) ────────────────────────
    //
    // The gap flagged at the end of Stage 2: every bulk read that existed
    // before this (`get_component_x_values`, `get_f32x2_interleaved`)
    // returns values only, with no way to know which entity each value
    // belongs to. Logic lives in `World::query_entity_ids` /
    // `query_first_id` / `query_match_count` (plain Rust, unit-tested
    // there) — these are just the `Uint32Array` marshaling on top.

    /// Entity ids matching ALL given components, as interleaved
    /// `[id0, generation0, id1, generation1, ...]`.
    pub fn query_entities(&self, component_ids: &[u32]) -> Uint32Array {
        let pairs = self.inner.query_entity_ids(component_ids);
        let mut flat: Vec<u32> = Vec::with_capacity(pairs.len() * 2);
        for (id, generation) in pairs {
            flat.push(id);
            flat.push(generation);
        }
        Uint32Array::from(flat.as_slice())
    }

    /// First matching entity as `[id, generation]`, or a zero-length array
    /// if nothing matches. `World::query_each` has no early-exit
    /// mechanism, so `query_first_id` still scans every match internally —
    /// fine at indie-2D scale; a true short-circuit would need that added
    /// to `query_each` itself, not a new method shape here.
    pub fn query_first(&self, component_ids: &[u32]) -> Uint32Array {
        match self.inner.query_first_id(component_ids) {
            Some((id, generation)) => Uint32Array::from(&[id, generation][..]),
            None => Uint32Array::new_with_length(0),
        }
    }

    /// Counts entities matching ALL given components without allocating an
    /// id array.
    pub fn query_count(&self, component_ids: &[u32]) -> u32 {
        self.inner.query_match_count(component_ids) as u32
    }

    // ── Bulk data access (zero-copy via TypedArrays) ──────────────────────

    /// Get all X values (field offset 0) of a component as Float32Array.
    /// Ideal for: reading positions to render (upload to GPU buffer).
    pub fn get_component_x_values(&self, component_id: u32) -> Float32Array {
        let buf = self.inner.get_flat_f32_buffer(component_id);
        let arr = Float32Array::new_with_length(buf.len() as u32);
        arr.copy_from(&buf);
        arr
    }

    /// Write X values (field offset 0) back into component data.
    /// Ideal for: physics results → ECS (bulk update from JS physics lib).
    pub fn set_component_x_values(&mut self, component_id: u32, values: &Float32Array) {
        let vals: Vec<f32> = values.to_vec();
        self.inner.set_flat_f32_buffer(component_id, 0, &vals);
    }

    /// Write Y values (field offset 4) back into component data.
    pub fn set_component_y_values(&mut self, component_id: u32, values: &Float32Array) {
        let vals: Vec<f32> = values.to_vec();
        self.inner.set_flat_f32_buffer(component_id, 4, &vals);
    }

    /// Get interleaved [x0, y0, x1, y1, ...] for a 2×f32 component.
    /// Useful for uploading directly to WebGL vertex buffer.
    pub fn get_f32x2_interleaved(&self, component_id: u32) -> Float32Array {
        let schema_size = 8usize; // 2×f32
        let mut out = Vec::new();

        for archetype in &self.inner.archetypes {
            if !archetype.mask.contains(component_id as usize) {
                continue;
            }
            let col_idx = archetype.column_index(component_id);
            for chunk in &archetype.chunks {
                let data = &chunk.columns[col_idx].data;
                for i in 0..chunk.len() {
                    let start = i * schema_size;
                    let x = f32::from_le_bytes(data[start..start + 4].try_into().unwrap());
                    let y = f32::from_le_bytes(data[start + 4..start + 8].try_into().unwrap());
                    out.push(x);
                    out.push(y);
                }
            }
        }

        let arr = Float32Array::new_with_length(out.len() as u32);
        arr.copy_from(&out);
        arr
    }

    // ── Single entity read/write (legacy, kept for compatibility) ─────────

    pub fn get_x(&self, entity_id: u32, generation: u32, component_id: u32) -> f32 {
        let e = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .get_f32x2(e, component_id)
            .map(|v| v.x())
            .unwrap_or(0.0)
    }

    pub fn get_y(&self, entity_id: u32, generation: u32, component_id: u32) -> f32 {
        let e = crate::Entity {
            id: entity_id,
            generation,
        };
        self.inner
            .get_f32x2(e, component_id)
            .map(|v| v.y())
            .unwrap_or(0.0)
    }

    pub fn set_xy(&mut self, entity_id: u32, generation: u32, component_id: u32, x: f32, y: f32) {
        let e = crate::Entity {
            id: entity_id,
            generation,
        };
        let _ = self.inner.set_component(
            e,
            ComponentInstance::new(
                component_id,
                vec![ComponentValue::F32(x), ComponentValue::F32(y)],
            ),
        );
    }

    // ── Built-in physics step (velocity integration) ───────────────────

    /// Apply velocity to position for all entities that have both components.
    /// dt: delta time in seconds
    /// This runs entirely in Rust/WASM — zero JS↔WASM copies.
    pub fn step_physics(&mut self, pos_id: u32, vel_id: u32, dt: f32) {
        let query = self.inner.query(&[pos_id, vel_id]);

        // Collect updates to avoid borrow issues
        let mut updates: Vec<(usize, usize, f32, f32)> = Vec::new(); // (arch_idx, chunk_row, new_x, new_y)

        // We need to iterate archetypes directly for in-place mutation
        for archetype in &self.inner.archetypes {
            if !archetype.mask.contains(pos_id as usize) {
                continue;
            }
            if !archetype.mask.contains(vel_id as usize) {
                continue;
            }

            let pos_col = archetype.column_index(pos_id);
            let vel_col = archetype.column_index(vel_id);

            for chunk in &archetype.chunks {
                let pos_data = &chunk.columns[pos_col].data;
                let vel_data = &chunk.columns[vel_col].data;

                for i in 0..chunk.len() {
                    let ps = i * 8;
                    let vs = i * 8;
                    let px = f32::from_le_bytes(pos_data[ps..ps + 4].try_into().unwrap());
                    let py = f32::from_le_bytes(pos_data[ps + 4..ps + 8].try_into().unwrap());
                    let vx = f32::from_le_bytes(vel_data[vs..vs + 4].try_into().unwrap());
                    let vy = f32::from_le_bytes(vel_data[vs + 4..vs + 8].try_into().unwrap());
                    updates.push((archetype.id as usize, i, px + vx * dt, py + vy * dt));
                }
            }
        }
        drop(query);

        // Apply updates
        for (arch_id, row, nx, ny) in updates {
            let archetype = &mut self.inner.archetypes[arch_id];
            let pos_col = archetype.column_index(pos_id);
            // row is global across chunks — find the right chunk
            let mut remaining = row;
            for chunk in &mut archetype.chunks {
                if remaining < chunk.len() {
                    let data = &mut chunk.columns[pos_col].data;
                    let start = remaining * 8;
                    data[start..start + 4].copy_from_slice(&nx.to_le_bytes());
                    data[start + 4..start + 8].copy_from_slice(&ny.to_le_bytes());
                    break;
                }
                remaining -= chunk.len();
            }
        }
    }

    // ── Spatial broadphase (uniform grid) ──────────────────────────────
    //
    // Phase 2 of the WASM/ECS migration plan
    // (`.tools/docs/engineering/WASM_ECS_MIGRATION_PLAN.md`) — ports the JS
    // game's own `spatialGrid`/`rebuildSpatialGrid`/`forEachEnemyNear`
    // (`index.html` ~line 5178). Logic lives in `SpatialGrid` (plain Rust,
    // unit-tested in `tests/spatial.rs`, no `wasm` feature needed) —
    // exactly the same layering as `query_entity_ids`/`query_first_id`
    // above; these methods are thin marshaling on top.
    //
    // `rebuild_spatial_index` is a plain method JS calls once per frame
    // (after physics, before anything queries proximity), matching
    // `step_physics`'s existing pattern rather than registering a
    // `System` into the `PostPhysics` stage — today nothing in this crate
    // actually drives the `Schedule` for real game systems (it's only
    // exercised by test-only systems in `tests/scheduler_stages.rs`); the
    // one real per-frame game step that exists, `step_physics`, is called
    // directly by JS, so a second, different registration mechanism here
    // would be an inconsistency, not an improvement.

    /// Create (or reset/resize) the broadphase grid for a given
    /// position-shaped (2×f32, x/y) component id, covering a
    /// `[0, world_w] x [0, world_h]` world. Call once before the first
    /// `rebuild_spatial_index`/`query_near`/`query_near_count` for this
    /// component id. Safe to call again later (e.g. if world bounds
    /// change) — it simply replaces the existing grid for that id.
    ///
    /// World dimensions are a caller-supplied parameter, not a crate
    /// constant — this crate is otherwise game-agnostic, so this specific
    /// game's world size has no business being hardcoded here.
    ///
    /// ```js
    /// world.init_spatial_grid(posId, 4600, 4600);
    /// ```
    pub fn init_spatial_grid(&mut self, component_id: u32, world_w: f32, world_h: f32) {
        self.spatial_grids
            .insert(component_id, SpatialGrid::new(CELL_SIZE, world_w, world_h));
    }

    /// Repopulate the `component_id` grid from every entity currently
    /// holding that component, reading its first two f32 fields as (x, y).
    /// No-op if `init_spatial_grid` was never called for this component id
    /// — same leniency as `run_stage`'s unknown-stage no-op.
    pub fn rebuild_spatial_index(&mut self, component_id: u32) {
        if !self.spatial_grids.contains_key(&component_id) {
            return;
        }
        let mut entries: Vec<(crate::Entity, f32, f32)> = Vec::new();
        self.inner.query_each(&[component_id], |row| {
            let x = row.read_f32(0, 0);
            let y = row.read_f32(0, 4);
            entries.push((row.entity, x, y));
        });
        // Safe to unwrap: presence just checked above, and nothing between
        // here and there can remove the entry.
        self.spatial_grids
            .get_mut(&component_id)
            .unwrap()
            .rebuild(entries);
    }

    /// Entities within `radius` of `(x, y)` in the `component_id` grid, as
    /// interleaved `[id0, generation0, id1, generation1, ...]` — same
    /// return shape as `query_entities`. Empty array if no grid has been
    /// initialized for this component id yet.
    pub fn query_near(&self, x: f32, y: f32, radius: f32, component_id: u32) -> Uint32Array {
        let grid = match self.spatial_grids.get(&component_id) {
            Some(g) => g,
            None => return Uint32Array::new_with_length(0),
        };
        let mut flat: Vec<u32> = Vec::new();
        for entity in grid.query_near(x, y, radius) {
            flat.push(entity.id);
            flat.push(entity.generation);
        }
        Uint32Array::from(flat.as_slice())
    }

    /// Count of entities within `radius` of `(x, y)` in the `component_id`
    /// grid, without allocating an id array — matches `query_count`'s
    /// allocation-free convention. `0` if no grid has been initialized for
    /// this component id yet.
    pub fn query_near_count(&self, x: f32, y: f32, radius: f32, component_id: u32) -> u32 {
        match self.spatial_grids.get(&component_id) {
            Some(g) => g.query_near_count(x, y, radius) as u32,
            None => 0,
        }
    }

    // ── Chase AI (WASM_ECS_MIGRATION_PLAN.md Phase 3 step 4) ────────────────
    //
    // Mirrors index.html's updateEnemies() "chase" brain velocity formula
    // exactly — rubber-band catch-up (effective speed ramps from the
    // enemy's own base speed toward the player's CURRENT speed, +8%, over
    // 30s of failed pursuit) plus predictive intercept (aim 0.35s ahead of
    // the player's heading instead of their current position). Deliberately
    // scalar, pure math, no ECS/ChaseTime component — the real "chase"
    // brain interleaves this with mutation hooks (mutVampiric/mutBruiser),
    // contact damage, the anti-kiting leash teleport, and AI-tick
    // throttling, none of which belong in the engine crate yet. JS keeps
    // owning e.chaseTime's persistence across frames and all of the above;
    // this only replaces the mechanical "given these inputs, what's the
    // velocity" formula, one call per enemy, diagnostic-only for now (see
    // index.html's syncWasmChaseSeekCheck).

    /// Returns `[vx, vy]` as a 2-element Float32Array. `chase_time` is the
    /// caller-owned seconds-of-failed-pursuit accumulator (JS's
    /// `e.chaseTime`); `player_vx/vy` feed the predictive-intercept lead.
    pub fn chase_seek_velocity(
        &self,
        ex: f32, ey: f32, espeed: f32, chase_time: f32,
        px: f32, py: f32, player_vx: f32, player_vy: f32, player_move_speed: f32,
    ) -> Float32Array {
        let catch_up = (chase_time / 30.0).min(1.0);
        let eff_speed = espeed + (espeed.max(player_move_speed * 1.08) - espeed) * catch_up;
        const LEAD_T: f32 = 0.35;
        let tx = px + player_vx * LEAD_T;
        let ty = py + player_vy * LEAD_T;
        let pdx = tx - ex;
        let pdy = ty - ey;
        let pd_raw = (pdx * pdx + pdy * pdy).sqrt();
        let pd = if pd_raw == 0.0 { 1.0 } else { pd_raw }; // matches JS's `Math.hypot(...)||1`
        let arr = Float32Array::new_with_length(2);
        arr.copy_from(&[(pdx / pd) * eff_speed, (pdy / pd) * eff_speed]);
        arr
    }

    /// "keep_distance_shoot" brain (ranged-role enemies) — mirrors
    /// index.html's updateEnemies() formula exactly: move away if closer
    /// than `desired-20`, toward if farther than `desired+20`, hold still
    /// in the dead zone between. Stateless — no accumulator like
    /// `chase_seek_velocity`'s `chase_time`, every call is fully determined
    /// by its arguments. Returns `[vx, vy]`.
    pub fn keep_distance_velocity(
        &self,
        ex: f32, ey: f32, espeed: f32, desired: f32,
        px: f32, py: f32,
    ) -> Float32Array {
        let dx = px - ex;
        let dy = py - ey;
        let d_raw = (dx * dx + dy * dy).sqrt();
        let d = if d_raw == 0.0 { 1.0 } else { d_raw };
        let (mut mx, mut my) = (0.0_f32, 0.0_f32);
        if d < desired - 20.0 {
            mx = -(dx / d);
            my = -(dy / d);
        } else if d > desired + 20.0 {
            mx = dx / d;
            my = dy / d;
        }
        let arr = Float32Array::new_with_length(2);
        arr.copy_from(&[mx * espeed, my * espeed]);
        arr
    }

    /// "swarm" brain (bat/rat_swarm/etc.) — orbits a slowly-breathing radius
    /// around the player, mirrors index.html's formula exactly. `swarm_phase`
    /// is the caller-owned accumulator (JS's `e.swarmPhase`, advanced by
    /// `dt*3` every frame regardless of this call) — stateless here just
    /// like `chase_seek_velocity`'s `chase_time`. Returns `[vx, vy]`.
    pub fn swarm_orbit_velocity(
        &self,
        ex: f32, ey: f32, espeed: f32, swarm_phase: f32,
        px: f32, py: f32,
    ) -> Float32Array {
        let orbit_r = 90.0 + (swarm_phase * 0.6).sin() * 40.0;
        let tx = px + swarm_phase.cos() * orbit_r;
        let ty = py + swarm_phase.sin() * orbit_r;
        let ddx = tx - ex;
        let ddy = ty - ey;
        let dd_raw = (ddx * ddx + ddy * ddy).sqrt();
        let dd = if dd_raw == 0.0 { 1.0 } else { dd_raw };
        let arr = Float32Array::new_with_length(2);
        arr.copy_from(&[(ddx / dd) * espeed, (ddy / dd) * espeed]);
        arr
    }

    // ── Stats ─────────────────────────────────────────────────────────────

    pub fn archetype_count(&self) -> u32 {
        self.inner.archetype_count() as u32
    }

    /// Returns a JSON string with world stats (for Vue devtools panel)
    pub fn stats_json(&self) -> String {
        format!(
            r#"{{"entities":{},"archetypes":{},"components":{}}}"#,
            self.inner.entity_count(),
            self.inner.archetype_count(),
            self.inner.component_registry.count(),
        )
    }

    // ── Generic raw component access (any schema, any field layout) ─────
    //
    // `add_component_f32x2/f32/u32/bool` only ever covered their own named
    // shape — there was no way to attach a component registered via
    // `register_schema` (e.g. an `Owner`/`Parent`/`Target` relation:
    // "id:u32,generation:u32") to an entity at all, even though it could
    // be *registered* since Stage 1. Stage 4 needs exactly this for
    // Transform's `Parent` and Projectile's `Owner` — both are plain
    // EntityRef relations (see the Relational Entity Pattern in
    // engine-architecture-stage1.md, section 0), so this is the one
    // generic primitive that unblocks all of them, rather than adding a
    // new named method per relation.

    /// Attach (or update in place, if already present) a component with
    /// an arbitrary field layout. `bytes.len()` must exactly equal the
    /// registered component's byte size (as returned by
    /// `get_component_schema`) — returns `false` and does nothing
    /// otherwise, same leniency as the rest of this API.
    pub fn add_component_raw(&mut self, entity_id: u32, generation: u32, component_id: u32, bytes: &[u8]) -> bool {
        let entity = crate::Entity { id: entity_id, generation };
        let schema = match self.inner.component_registry.get(component_id) {
            Some(s) => s,
            None => return false,
        };
        if bytes.len() != schema.size {
            return false;
        }
        let values = schema.deserialize(bytes);
        self.inner.add_component(entity, ComponentInstance::new(component_id, values)).is_ok()
    }

    /// Read a component's raw bytes back, in field order, matching its
    /// registered schema. Empty array if the entity is dead or lacks the
    /// component.
    pub fn get_component_raw(&self, entity_id: u32, generation: u32, component_id: u32) -> Vec<u8> {
        let entity = crate::Entity { id: entity_id, generation };
        let schema = match self.inner.component_registry.get(component_id) {
            Some(s) => s,
            None => return Vec::new(),
        };
        match self.inner.get_component(entity, component_id) {
            Some(values) => schema.serialize(&values),
            None => Vec::new(),
        }
    }

    /// Overwrite an already-present component's raw bytes in place.
    /// Returns `false` (no-op) if the entity is dead, lacks the
    /// component, or `bytes.len()` doesn't match the schema size.
    pub fn set_component_raw(&mut self, entity_id: u32, generation: u32, component_id: u32, bytes: &[u8]) -> bool {
        let entity = crate::Entity { id: entity_id, generation };
        let schema = match self.inner.component_registry.get(component_id) {
            Some(s) => s,
            None => return false,
        };
        if bytes.len() != schema.size {
            return false;
        }
        let values = schema.deserialize(bytes);
        self.inner.set_component(entity, ComponentInstance::new(component_id, values)).is_ok()
    }

    // ── Generic schema registration ─────────────────────────────────────
    //
    // The five `register_f32x2/f32/u32/bool/f32x3` helpers above only cover
    // the shapes they're named after. `spec` lets JS register *any*
    // mixed-type, multi-field shape — e.g. an EntityRef-style relation
    // (`"id:u32,generation:u32"`) or a 4-field Stats block
    // (`"base:f32,flat:f32,percent:f32,multiplier:f32"`) — without ever
    // needing a new Rust method. `ComponentBuilder`/`FieldType` already
    // supported this internally; this just finally exposes it.
    //
    // `spec` is a comma-separated `name:type` list. Supported types:
    // f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool.

    /// Register a component with an arbitrary, mixed-type field list.
    ///
    /// ```js
    /// const ownerId = world.register_schema("Owner", "id:u32,generation:u32");
    ///
    /// const statsId = world.register_schema(
    ///   "Stats", "base:f32,flat:f32,percent:f32,multiplier:f32"
    /// );
    /// ```
    pub fn register_schema(&mut self, name: &str, spec: &str) -> u32 {
        self.inner.register_component(parse_schema(name, spec).build())
    }

    // ── Resources (singleton, non-entity data) ─────────────────────────

    /// Register a resource with an arbitrary field list (same `spec` syntax
    /// as `register_schema`). e.g. `register_resource("Time", "elapsed:f32,delta:f32")`.
    pub fn register_resource(&mut self, name: &str, spec: &str) -> u32 {
        self.inner.register_resource(parse_schema(name, spec).build())
    }

    pub fn resource_id(&self, name: &str) -> u32 {
        self.inner.resource_id(name).unwrap_or(u32::MAX)
    }

    pub fn has_resource(&self, resource_id: u32) -> bool {
        self.inner.resources.has(resource_id)
    }

    /// Get a resource schema as a JSON string (same shape as
    /// `get_component_schema`). Returns `"null"` if not registered.
    pub fn get_resource_schema(&self, resource_id: u32) -> String {
        self.inner
            .resources
            .schema(resource_id)
            .map(|s| s.describe())
            .unwrap_or_else(|| "null".to_string())
    }

    /// Read a single f32 field at `byte_offset` within a resource.
    /// Returns 0.0 if the resource isn't registered.
    pub fn get_resource_f32(&self, resource_id: u32, byte_offset: usize) -> f32 {
        self.inner
            .resources
            .get_bytes(resource_id)
            .and_then(|b| {
                if byte_offset + 4 <= b.len() {
                    Some(f32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap()))
                } else {
                    None
                }
            })
            .unwrap_or(0.0)
    }

    /// Write a single f32 field at `byte_offset` within a resource in-place.
    pub fn set_resource_f32(&mut self, resource_id: u32, byte_offset: usize, value: f32) {
        if let Some(b) = self.inner.resources.get_bytes_mut(resource_id) {
            if byte_offset + 4 <= b.len() {
                b[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
    }

    /// Read a single u32 field at `byte_offset` within a resource.
    pub fn get_resource_u32(&self, resource_id: u32, byte_offset: usize) -> u32 {
        self.inner
            .resources
            .get_bytes(resource_id)
            .and_then(|b| {
                if byte_offset + 4 <= b.len() {
                    Some(u32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap()))
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    /// Write a single u32 field at `byte_offset` within a resource in-place.
    pub fn set_resource_u32(&mut self, resource_id: u32, byte_offset: usize, value: u32) {
        if let Some(b) = self.inner.resources.get_bytes_mut(resource_id) {
            if byte_offset + 4 <= b.len() {
                b[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
            }
        }
    }

    // ── Events (frame-scoped queues) ────────────────────────────────────
    //
    // Events are produced almost exclusively by Rust systems (Stage 3) and
    // consumed by JS (Audio/Animation/Vue), so the JS-facing surface here
    // is read-oriented: register the shape once, then drain once per frame.
    // There's intentionally no generic "send an arbitrary event from JS"
    // method yet — if a concrete JS-originated event turns out to be needed
    // (e.g. a UI-triggered DialogueEvent), add one narrow method for that
    // specific shape then, the same way `step_physics` is one narrow method
    // rather than a generic system-runner.

    /// Register an event type with an arbitrary field list (same `spec`
    /// syntax as `register_schema`).
    pub fn register_event(&mut self, name: &str, spec: &str) -> u32 {
        self.inner.register_event(parse_schema(name, spec).build())
    }

    pub fn event_id(&self, name: &str) -> u32 {
        self.inner.event_id(name).unwrap_or(u32::MAX)
    }

    pub fn get_event_schema(&self, event_id: u32) -> String {
        self.inner
            .events
            .schema(event_id)
            .map(|s| s.describe())
            .unwrap_or_else(|| "null".to_string())
    }

    pub fn event_count(&self, event_id: u32) -> u32 {
        self.inner.events.count(event_id) as u32
    }

    /// Read and clear this frame's queued events of the given type, as a
    /// JSON array of field-name → value objects, e.g.
    /// `[{"target_id":5,"target_generation":0,"amount":12.5}]`.
    /// Call once per event type, once per frame, after running the
    /// schedule for that frame.
    pub fn drain_events_json(&mut self, event_id: u32) -> String {
        let schema = match self.inner.events.schema(event_id) {
            Some(s) => s.clone(),
            None => return "[]".to_string(),
        };
        let drained = self.inner.events.drain(event_id);
        let items: Vec<String> = drained
            .iter()
            .map(|values| {
                let fields: Vec<String> = schema
                    .fields
                    .iter()
                    .zip(values.iter())
                    .map(|(f, v)| format!("\"{}\":{}", f.name, component_value_to_json(v)))
                    .collect();
                format!("{{{}}}", fields.join(","))
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    // ── Schedule ─────────────────────────────────────────────────────────
    //
    // `World` always starts with the twelve fixed pipeline stages (Startup/
    // PreFrame/Input/PrePhysics/Physics/PostPhysics/AI/Gameplay/Animation/
    // Audio/Render/PostFrame — see `Schedule::standard_game_schedule`).
    // Rust game systems register themselves into these via
    // `Schedule::add_system_to` from native Rust code; JS never builds or
    // reorders the schedule — it only ticks it, per the layering rule that
    // no upper layer knows the internals of the layer below it.

    /// Run only the named stage's systems once.
    pub fn run_stage(&mut self, stage_name: &str) {
        self.inner.run_stage(stage_name);
    }

    /// Run every stage once, in order — one full simulation tick.
    pub fn run_schedule(&mut self) {
        self.inner.run_schedule();
    }

    /// Returns the configured stage names, in execution order, as a JSON
    /// array of strings — mainly for a Vue devtools/inspector panel.
    pub fn stage_names(&self) -> String {
        let names = self.inner.stage_names();
        format!(
            "[{}]",
            names.iter().map(|n| format!("\"{}\"", n)).collect::<Vec<_>>().join(",")
        )
    }
}

/// Parses a `"name:type,name:type,..."` spec into a `ComponentBuilder`.
/// Shared by `register_schema`, `register_resource`, and `register_event` —
/// a resource and an event are, structurally, just a component schema used
/// in a different storage context (see `Resources`/`Events`).
///
/// Deliberately a free function, *outside* the `#[wasm_bindgen] impl
/// JsWorld` block: it has no `self` and isn't part of the public JS API,
/// and keeping it out of the annotated impl block avoids any reliance on
/// how the wasm-bindgen macro treats non-`pub` items inside an impl it
/// processes.
fn parse_schema(name: &str, spec: &str) -> ComponentBuilder {
    let mut builder = ComponentBuilder::new(name);
    for field in spec.split(',') {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        let mut parts = field.splitn(2, ':');
        let field_name = parts.next().unwrap_or("").trim();
        let type_name = parts.next().unwrap_or("f32").trim();
        let field_type = match type_name {
            "f32" => FieldType::F32,
            "f64" => FieldType::F64,
            "i8" => FieldType::I8,
            "i16" => FieldType::I16,
            "i32" => FieldType::I32,
            "i64" => FieldType::I64,
            "u8" => FieldType::U8,
            "u16" => FieldType::U16,
            "u32" => FieldType::U32,
            "u64" => FieldType::U64,
            "bool" => FieldType::Bool,
            other => panic!(
                "register_schema: unknown field type '{}' for field '{}' in component '{}'",
                other, field_name, name
            ),
        };
        builder = builder.field(field_name, field_type);
    }
    builder
}

/// Minimal `ComponentValue` → JSON literal formatter (no external JSON
/// dependency, consistent with `ComponentSchema::describe`/`stats_json`
/// elsewhere in this file, which hand-roll JSON the same way).
fn component_value_to_json(value: &ComponentValue) -> String {
    match value {
        ComponentValue::F32(v) => format!("{}", v),
        ComponentValue::F64(v) => format!("{}", v),
        ComponentValue::I8(v) => format!("{}", v),
        ComponentValue::I16(v) => format!("{}", v),
        ComponentValue::I32(v) => format!("{}", v),
        ComponentValue::I64(v) => format!("{}", v),
        ComponentValue::U8(v) => format!("{}", v),
        ComponentValue::U16(v) => format!("{}", v),
        ComponentValue::U32(v) => format!("{}", v),
        ComponentValue::U64(v) => format!("{}", v),
        ComponentValue::Bool(v) => format!("{}", v),
    }
}
