/// Entity handle — generational index for O(1) validity checks
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl Entity {
    pub const INVALID: Self = Self {
        id: u32::MAX,
        generation: u32::MAX,
    };
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({}, gen={})", self.id, self.generation)
    }
}

/// Manages entity lifecycle with generational recycling
pub struct EntityManager {
    generations: Vec<u32>,
    free_ids: Vec<u32>,
    alive_count: u32,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_ids: Vec::new(),
            alive_count: 0,
        }
    }

    pub fn create(&mut self) -> Entity {
        self.alive_count += 1;
        if let Some(id) = self.free_ids.pop() {
            Entity {
                id,
                generation: self.generations[id as usize],
            }
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            Entity { id, generation: 0 }
        }
    }

    pub fn destroy(&mut self, entity: Entity) {
        let idx = entity.id as usize;
        if idx >= self.generations.len() {
            return;
        }
        if self.generations[idx] != entity.generation {
            return; // already dead
        }
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free_ids.push(entity.id);
        if self.alive_count > 0 {
            self.alive_count -= 1;
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.generations
            .get(entity.id as usize)
            .map(|g| *g == entity.generation)
            .unwrap_or(false)
    }

    pub fn alive_count(&self) -> u32 {
        self.alive_count
    }

    pub fn total_count(&self) -> u32 {
        self.generations.len() as u32
    }

    /// Returns the current generation for a raw entity ID.
    /// Used by the WASM API to reconstruct a full Entity handle after a bare-ID spawn.
    /// Returns `u32::MAX` if `id` is out of range.
    pub fn generation_of(&self, id: u32) -> u32 {
        self.generations
            .get(id as usize)
            .copied()
            .unwrap_or(u32::MAX)
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}
