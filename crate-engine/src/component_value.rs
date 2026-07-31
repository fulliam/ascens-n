/// A dynamically-typed component field value
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentValue {
    F32(f32),
    F64(f64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    Bool(bool),
}

impl ComponentValue {
    pub fn as_f32(&self) -> Option<f32> {
        if let ComponentValue::F32(v) = self { Some(*v) } else { None }
    }
    pub fn as_f64(&self) -> Option<f64> {
        if let ComponentValue::F64(v) = self { Some(*v) } else { None }
    }
    pub fn as_i32(&self) -> Option<i32> {
        if let ComponentValue::I32(v) = self { Some(*v) } else { None }
    }
    pub fn as_u32(&self) -> Option<u32> {
        if let ComponentValue::U32(v) = self { Some(*v) } else { None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        if let ComponentValue::Bool(v) = self { Some(*v) } else { None }
    }
}
