use hashbrown::HashMap;
use crate::component::ComponentSchema;

/// Stores all registered component schemas
pub struct ComponentRegistry {
    schemas: Vec<ComponentSchema>,
    name_to_id: HashMap<String, u32>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self { schemas: Vec::new(), name_to_id: HashMap::new() }
    }

    pub fn register(&mut self, mut schema: ComponentSchema) -> u32 {
        if let Some(&id) = self.name_to_id.get(&schema.name) {
            return id;
        }
        let id = self.schemas.len() as u32;
        schema.id = id;
        self.name_to_id.insert(schema.name.clone(), id);
        self.schemas.push(schema);
        id
    }

    pub fn register_schema(&mut self, schema: ComponentSchema) -> u32 {
        self.register(schema)
    }

    pub fn get(&self, id: u32) -> Option<&ComponentSchema> {
        self.schemas.get(id as usize)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ComponentSchema> {
        self.name_to_id.get(name).and_then(|&id| self.get(id))
    }

    pub fn count(&self) -> usize {
        self.schemas.len()
    }

    pub fn all_schemas(&self) -> &[ComponentSchema] {
        &self.schemas
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
