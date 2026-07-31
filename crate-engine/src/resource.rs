use crate::{ComponentRegistry, ComponentSchema, ComponentValue, EcsError, EcsResult};

/// Global, non-entity singleton data: Time, Input, Camera, GameState, ...
///
/// A Resource is implemented as a single-instance component: it reuses the
/// exact same `ComponentSchema` / serialize / deserialize machinery as
/// regular components, just stored once (no archetype, no entity, no
/// per-row storage) instead of once-per-entity. This keeps the engine to
/// one schema/serialization concept instead of two.
pub struct Resources {
    registry: ComponentRegistry,
    data: hashbrown::HashMap<u32, Vec<u8>>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            data: hashbrown::HashMap::new(),
        }
    }

    /// Register a resource schema. Idempotent by name (same behavior as
    /// component registration) — calling this twice with the same name
    /// returns the same id and does not reset the stored value.
    pub fn register(&mut self, schema: ComponentSchema) -> u32 {
        let size = schema.size;
        let id = self.registry.register_schema(schema);
        self.data.entry(id).or_insert_with(|| vec![0u8; size]);
        id
    }

    pub fn resource_id(&self, name: &str) -> Option<u32> {
        self.registry.get_by_name(name).map(|s| s.id)
    }

    pub fn has(&self, resource_id: u32) -> bool {
        self.data.contains_key(&resource_id)
    }

    pub fn schema(&self, resource_id: u32) -> Option<&ComponentSchema> {
        self.registry.get(resource_id)
    }

    /// Overwrite the resource's value (all fields at once, in schema order).
    pub fn set(&mut self, resource_id: u32, values: &[ComponentValue]) -> EcsResult<()> {
        let schema = self
            .registry
            .get(resource_id)
            .ok_or(EcsError::ResourceNotRegistered(resource_id))?;
        let bytes = schema.serialize(values);
        self.data.insert(resource_id, bytes);
        Ok(())
    }

    pub fn get(&self, resource_id: u32) -> Option<Vec<ComponentValue>> {
        let schema = self.registry.get(resource_id)?;
        let bytes = self.data.get(&resource_id)?;
        Some(schema.deserialize(bytes))
    }

    /// Raw byte access — used by the WASM layer for cheap single-field
    /// get/set (e.g. reading just `DeltaTime` out of a `Time` resource)
    /// without round-tripping the whole `Vec<ComponentValue>`.
    pub fn get_bytes(&self, resource_id: u32) -> Option<&[u8]> {
        self.data.get(&resource_id).map(|v| v.as_slice())
    }

    pub fn get_bytes_mut(&mut self, resource_id: u32) -> Option<&mut [u8]> {
        self.data.get_mut(&resource_id).map(|v| v.as_mut_slice())
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}
