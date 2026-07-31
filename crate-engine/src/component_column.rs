use crate::{ComponentSchema, FieldColumn};

/// Higher-level column grouping all fields of one component (for query iteration)
pub struct ComponentColumn {
    pub component_id: u32,
    pub fields: Vec<FieldColumn>,
}

impl ComponentColumn {
    pub fn new(component_id: u32, fields: Vec<FieldColumn>) -> Self {
        Self { component_id, fields }
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.field_name == name)
    }

    pub fn from_schema(schema: &ComponentSchema) -> Self {
        let fields = schema.fields
            .iter()
            .map(|f| FieldColumn::new(f.name.clone(), f.field_type))
            .collect();
        Self { component_id: schema.id, fields }
    }
}
