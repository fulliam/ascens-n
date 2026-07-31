use hashbrown::HashMap;

/// Maps sorted component ID sets → archetype IDs for O(1) lookup
pub struct ArchetypeRegistry {
    lookup: HashMap<Vec<u32>, u32>,
}

impl ArchetypeRegistry {
    pub fn new() -> Self {
        Self { lookup: HashMap::new() }
    }

    pub fn insert(&mut self, components: Vec<u32>, archetype_id: u32) {
        self.lookup.insert(components, archetype_id);
    }

    pub fn get(&self, components: &[u32]) -> Option<u32> {
        self.lookup.get(components).copied()
    }

    pub fn count(&self) -> usize {
        self.lookup.len()
    }
}

impl Default for ArchetypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
