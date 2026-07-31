use crate::{archetype::Column, ColumnView, Entity};

/// One chunk's worth of query results
pub struct QueryChunk<'a> {
    pub rows: usize,
    pub columns: Vec<ColumnView<'a>>,
}

impl<'a> QueryChunk<'a> {
    pub fn column(&self, index: usize) -> &ColumnView<'a> {
        &self.columns[index]
    }
}

/// Full query result across all matching archetypes/chunks
pub struct QueryResult<'a> {
    pub chunks: Vec<QueryChunk<'a>>,
}

impl<'a> QueryResult<'a> {
    pub fn total_rows(&self) -> usize {
        self.chunks.iter().map(|c| c.rows).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.total_rows() == 0
    }

    /// Iterate all rows across all chunks, yielding (chunk_ref, row_index).
    pub fn iter_rows(&self) -> impl Iterator<Item = (&QueryChunk<'a>, usize)> {
        self.chunks
            .iter()
            .flat_map(|chunk| (0..chunk.rows).map(move |row| (chunk, row)))
    }

    /// Alias for `iter_rows`.
    pub fn iter(&self) -> impl Iterator<Item = (&QueryChunk<'a>, usize)> {
        self.iter_rows()
    }

    /// Returns the single matching `(chunk, row)`, or `None` if zero or multiple match.
    ///
    /// Useful for unique entities (player, camera, game state).
    pub fn single(&self) -> Option<(&QueryChunk<'a>, usize)> {
        if self.total_rows() != 1 {
            return None;
        }
        self.iter_rows().next()
    }

    /// Returns the first matching `(chunk, row)`, or `None` if nothing matches.
    pub fn first(&self) -> Option<(&QueryChunk<'a>, usize)> {
        self.iter_rows().next()
    }

    /// Run a callback for every matching `(chunk, row)`.
    ///
    /// ```rust,ignore
    /// world.query(&[pos_id]).for_each(|chunk, row| {
    ///     let x = chunk.columns[0].read_f32_at(row, 0);
    /// });
    /// ```
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&QueryChunk<'a>, usize),
    {
        for (chunk, row) in self.iter_rows() {
            f(chunk, row);
        }
    }

    /// Run a callback for every row with direct access to the selected column views.
    ///
    /// `cols[i]` is the view for the i-th queried component (same order as the
    /// `component_ids` slice passed to `world.query()`).
    ///
    /// ```rust,ignore
    /// world.query(&[pos_id, vel_id]).for_each_row(|cols, row| {
    ///     let px = cols[0].read_f32_at(row, 0); // Position.x
    ///     let vx = cols[1].read_f32_at(row, 0); // Velocity.x
    /// });
    /// ```
    pub fn for_each_row<F>(&self, mut f: F)
    where
        F: FnMut(&[ColumnView<'a>], usize),
    {
        for chunk in &self.chunks {
            for row in 0..chunk.rows {
                f(&chunk.columns, row);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// QueryRow — used with World::query_each
// ─────────────────────────────────────────────────────────────────────────────

/// Read-only view of one entity's data across multiple query columns.
///
/// Obtained via [`World::query_each`]. Column indices correspond to the order of
/// `component_ids` passed to that method, not the raw archetype column layout.
///
/// Field byte offsets follow little-endian SoA packing:
/// - field 0 → offset 0
/// - field 1 → offset `size_of(field_0)` (e.g. 4 for f32)
/// - field 2 → offset `size_of(field_0) + size_of(field_1)`, etc.
pub struct QueryRow<'a> {
    pub(crate) columns: &'a [Column],
    pub(crate) col_indices: &'a [usize],
    /// The entity this row belongs to — needed to target `CommandBuffer`
    /// operations (despawn, add/remove/set component) at the right entity
    /// from inside a read-only query iteration.
    pub entity: Entity,
    /// Row within the current chunk.
    pub row: usize,
}

impl<'a> QueryRow<'a> {
    /// Raw byte slice for query column `col_idx`.
    #[inline]
    pub fn bytes(&self, col_idx: usize) -> &[u8] {
        self.columns[self.col_indices[col_idx]].row(self.row)
    }

    /// Read an `f32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_f32(&self, col_idx: usize, byte_offset: usize) -> f32 {
        let b = self.bytes(col_idx);
        f32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read a `u32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_u32(&self, col_idx: usize, byte_offset: usize) -> u32 {
        let b = self.bytes(col_idx);
        u32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read an `i32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_i32(&self, col_idx: usize, byte_offset: usize) -> i32 {
        let b = self.bytes(col_idx);
        i32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read a `bool` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_bool(&self, col_idx: usize, byte_offset: usize) -> bool {
        self.bytes(col_idx)[byte_offset] != 0
    }
}
