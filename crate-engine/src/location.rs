/// Where in the archetype storage an entity lives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityLocation {
    pub archetype_id: u32,
    pub chunk_index: u32,
    pub row: u32,
}

impl EntityLocation {
    pub const INVALID: Self = Self {
        archetype_id: u32::MAX,
        chunk_index: u32::MAX,
        row: u32::MAX,
    };

    pub fn is_valid(self) -> bool {
        self.archetype_id != u32::MAX
    }
}
