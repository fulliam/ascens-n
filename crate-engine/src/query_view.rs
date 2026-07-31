/// Read-only view into one column's data
pub struct ColumnView<'a> {
    pub data: &'a [u8],
    pub element_size: usize,
}

impl<'a> ColumnView<'a> {
    #[inline(always)]
    pub fn row(&self, index: usize) -> &'a [u8] {
        let start = index * self.element_size;
        &self.data[start..start + self.element_size]
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        if self.element_size == 0 { 0 } else { self.data.len() / self.element_size }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Read f32 at byte offset within a row
    #[inline(always)]
    pub fn read_f32_at(&self, row: usize, byte_offset: usize) -> f32 {
        let start = row * self.element_size + byte_offset;
        f32::from_le_bytes(self.data[start..start + 4].try_into().unwrap())
    }

    /// Iterate all rows as byte slices
    pub fn iter(&self) -> impl Iterator<Item = &[u8]> {
        (0..self.len()).map(move |i| self.row(i))
    }
}
