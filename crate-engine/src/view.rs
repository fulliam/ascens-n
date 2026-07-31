/// Type-safe view for 2xF32 components (Position, Velocity, etc.)
pub struct F32x2View<'a> {
    pub data: &'a [u8],
}

impl<'a> F32x2View<'a> {
    #[inline(always)]
    pub fn x(&self) -> f32 {
        f32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        f32::from_le_bytes(self.data[4..8].try_into().unwrap())
    }
}

/// Mutable view for 2xF32 components
pub struct F32x2ViewMut<'a> {
    pub data: &'a mut [u8],
}

impl<'a> F32x2ViewMut<'a> {
    #[inline(always)]
    pub fn x(&self) -> f32 {
        f32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    #[inline(always)]
    pub fn y(&self) -> f32 {
        f32::from_le_bytes(self.data[4..8].try_into().unwrap())
    }

    #[inline(always)]
    pub fn set_x(&mut self, v: f32) {
        self.data[0..4].copy_from_slice(&v.to_le_bytes());
    }

    #[inline(always)]
    pub fn set_y(&mut self, v: f32) {
        self.data[4..8].copy_from_slice(&v.to_le_bytes());
    }
}

/// Type-safe view for 3xF32 components (Position3D, etc.)
pub struct F32x3View<'a> {
    pub data: &'a [u8],
}

impl<'a> F32x3View<'a> {
    #[inline(always)]
    pub fn x(&self) -> f32 { f32::from_le_bytes(self.data[0..4].try_into().unwrap()) }
    #[inline(always)]
    pub fn y(&self) -> f32 { f32::from_le_bytes(self.data[4..8].try_into().unwrap()) }
    #[inline(always)]
    pub fn z(&self) -> f32 { f32::from_le_bytes(self.data[8..12].try_into().unwrap()) }
}
