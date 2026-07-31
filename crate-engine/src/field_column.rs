use crate::FieldType;

/// A per-field column for struct-of-arrays iteration
pub struct FieldColumn {
    pub field_name: String,
    pub field_type: FieldType,
    pub element_size: usize,
    pub data: Vec<u8>,
}

impl FieldColumn {
    pub fn new(field_name: String, field_type: FieldType) -> Self {
        Self {
            field_name,
            field_type,
            element_size: field_type.size(),
            data: Vec::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.element_size == 0 { 0 } else { self.data.len() / self.element_size }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Direct slice of all values (for SIMD / bulk copy to WASM)
    pub fn as_f32_slice(&self) -> Option<&[f32]> {
        if matches!(self.field_type, FieldType::F32) {
            // SAFETY: f32 is 4 bytes, data.len() is a multiple of 4, data is aligned to u8
            Some(unsafe {
                std::slice::from_raw_parts(
                    self.data.as_ptr() as *const f32,
                    self.data.len() / 4,
                )
            })
        } else {
            None
        }
    }
}
