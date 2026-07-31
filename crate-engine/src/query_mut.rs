use crate::{archetype::Column, Entity};

/// Mutable view of one entity's data across multiple query columns.
///
/// Obtained via [`World::query_each_mut`]. Column indices correspond to the
/// order of `component_ids` passed to that method.
///
/// You can freely mix reads and writes: read from a velocity column, write to
/// a position column — all safe, no `unsafe` required.
///
/// Field byte offsets follow little-endian SoA packing (same as [`QueryRow`]):
/// - field 0 → offset 0  
/// - field 1 → offset `size_of(field_0)` (e.g. 4 for f32)
pub struct QueryRowMut<'a> {
    pub(crate) columns: &'a mut Vec<Column>,
    pub(crate) col_indices: &'a [usize],
    /// The entity this row belongs to (see `QueryRow::entity`).
    pub entity: Entity,
    /// Row within the current chunk.
    pub row: usize,
}

impl<'a> QueryRowMut<'a> {
    // ── Reads ────────────────────────────────────────────────────────────────

    /// Read an `f32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_f32(&self, col_idx: usize, byte_offset: usize) -> f32 {
        let b = self.columns[self.col_indices[col_idx]].row(self.row);
        f32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read a `u32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_u32(&self, col_idx: usize, byte_offset: usize) -> u32 {
        let b = self.columns[self.col_indices[col_idx]].row(self.row);
        u32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read an `i32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_i32(&self, col_idx: usize, byte_offset: usize) -> i32 {
        let b = self.columns[self.col_indices[col_idx]].row(self.row);
        i32::from_le_bytes(b[byte_offset..byte_offset + 4].try_into().unwrap())
    }

    /// Read a `bool` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn read_bool(&self, col_idx: usize, byte_offset: usize) -> bool {
        self.columns[self.col_indices[col_idx]].row(self.row)[byte_offset] != 0
    }

    // ── Writes ───────────────────────────────────────────────────────────────

    /// Write an `f32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn write_f32(&mut self, col_idx: usize, byte_offset: usize, value: f32) {
        let bytes = self.columns[self.col_indices[col_idx]].row_mut(self.row);
        bytes[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Write a `u32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn write_u32(&mut self, col_idx: usize, byte_offset: usize, value: u32) {
        let bytes = self.columns[self.col_indices[col_idx]].row_mut(self.row);
        bytes[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Write an `i32` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn write_i32(&mut self, col_idx: usize, byte_offset: usize, value: i32) {
        let bytes = self.columns[self.col_indices[col_idx]].row_mut(self.row);
        bytes[byte_offset..byte_offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// Write a `bool` at `byte_offset` within query column `col_idx`.
    #[inline]
    pub fn write_bool(&mut self, col_idx: usize, byte_offset: usize, value: bool) {
        let bytes = self.columns[self.col_indices[col_idx]].row_mut(self.row);
        bytes[byte_offset] = if value { 1 } else { 0 };
    }
}
