use crate::ComponentValue;

/// Supported field types for component schemas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    F32, F64,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    Bool,
}

impl FieldType {
    #[inline]
    pub fn size(self) -> usize {
        match self {
            FieldType::F32 => 4, FieldType::F64 => 8,
            FieldType::I8  => 1, FieldType::I16 => 2, FieldType::I32 => 4, FieldType::I64 => 8,
            FieldType::U8  => 1, FieldType::U16 => 2, FieldType::U32 => 4, FieldType::U64 => 8,
            FieldType::Bool => 1,
        }
    }

    #[inline]
    pub fn write_value(self, value: &ComponentValue, output: &mut Vec<u8>) {
        match (self, value) {
            (FieldType::F32, ComponentValue::F32(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::F64, ComponentValue::F64(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::I8,  ComponentValue::I8(v))  => output.push(*v as u8),
            (FieldType::I16, ComponentValue::I16(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::I32, ComponentValue::I32(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::I64, ComponentValue::I64(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::U8,  ComponentValue::U8(v))  => output.push(*v),
            (FieldType::U16, ComponentValue::U16(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::U32, ComponentValue::U32(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::U64, ComponentValue::U64(v)) => output.extend_from_slice(&v.to_le_bytes()),
            (FieldType::Bool, ComponentValue::Bool(v)) => output.push(if *v { 1 } else { 0 }),
            _ => panic!("FieldType::write_value: type mismatch"),
        }
    }

    #[inline]
    pub fn read_value(self, bytes: &[u8]) -> ComponentValue {
        match self {
            FieldType::F32  => ComponentValue::F32(f32::from_le_bytes(bytes[..4].try_into().unwrap())),
            FieldType::F64  => ComponentValue::F64(f64::from_le_bytes(bytes[..8].try_into().unwrap())),
            FieldType::I8   => ComponentValue::I8(bytes[0] as i8),
            FieldType::I16  => ComponentValue::I16(i16::from_le_bytes(bytes[..2].try_into().unwrap())),
            FieldType::I32  => ComponentValue::I32(i32::from_le_bytes(bytes[..4].try_into().unwrap())),
            FieldType::I64  => ComponentValue::I64(i64::from_le_bytes(bytes[..8].try_into().unwrap())),
            FieldType::U8   => ComponentValue::U8(bytes[0]),
            FieldType::U16  => ComponentValue::U16(u16::from_le_bytes(bytes[..2].try_into().unwrap())),
            FieldType::U32  => ComponentValue::U32(u32::from_le_bytes(bytes[..4].try_into().unwrap())),
            FieldType::U64  => ComponentValue::U64(u64::from_le_bytes(bytes[..8].try_into().unwrap())),
            FieldType::Bool => ComponentValue::Bool(bytes[0] != 0),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            FieldType::F32 => "f32", FieldType::F64 => "f64",
            FieldType::I8  => "i8",  FieldType::I16 => "i16",
            FieldType::I32 => "i32", FieldType::I64 => "i64",
            FieldType::U8  => "u8",  FieldType::U16 => "u16",
            FieldType::U32 => "u32", FieldType::U64 => "u64",
            FieldType::Bool => "bool",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: FieldType,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentSchema {
    pub id: u32,
    pub name: String,
    pub size: usize,
    pub fields: Vec<FieldSchema>,
}

impl ComponentSchema {
    /// Serialize component values into contiguous bytes (SoA-ready)
    pub fn serialize(&self, values: &[ComponentValue]) -> Vec<u8> {
        assert_eq!(self.fields.len(), values.len(), "field count mismatch");
        let mut bytes = Vec::with_capacity(self.size);
        for (field, value) in self.fields.iter().zip(values.iter()) {
            field.field_type.write_value(value, &mut bytes);
        }
        debug_assert_eq!(bytes.len(), self.size);
        bytes
    }

    /// Deserialize bytes back into typed component values
    pub fn deserialize(&self, bytes: &[u8]) -> Vec<ComponentValue> {
        let mut result = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            let start = field.offset;
            let end = start + field.field_type.size();
            result.push(field.field_type.read_value(&bytes[start..end]));
        }
        result
    }

    /// Returns a JSON-compatible description for JS interop
    pub fn describe(&self) -> String {
        let fields: Vec<String> = self.fields
            .iter()
            .map(|f| format!("{{\"name\":\"{}\",\"type\":\"{}\",\"offset\":{}}}", 
                f.name, f.field_type.type_name(), f.offset))
            .collect();
        format!("{{\"id\":{},\"name\":\"{}\",\"size\":{},\"fields\":[{}]}}", 
            self.id, self.name, self.size, fields.join(","))
    }
}
