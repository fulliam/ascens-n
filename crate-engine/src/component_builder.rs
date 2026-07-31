use crate::{ComponentSchema, FieldSchema, FieldType};

/// Fluent builder for ComponentSchema
pub struct ComponentBuilder {
    name: String,
    fields: Vec<FieldSchema>,
    size: usize,
}

impl ComponentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), fields: Vec::new(), size: 0 }
    }

    pub fn field(mut self, name: impl Into<String>, field_type: FieldType) -> Self {
        let offset = self.size;
        self.fields.push(FieldSchema { name: name.into(), field_type, offset });
        self.size += field_type.size();
        self
    }

    pub fn build(self) -> ComponentSchema {
        ComponentSchema { id: 0, name: self.name, size: self.size, fields: self.fields }
    }
}
