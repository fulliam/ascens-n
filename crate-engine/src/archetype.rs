use crate::{CHUNK_CAPACITY, ComponentMask, Entity};
use hashbrown::HashMap;

/// Raw SoA column — contiguous bytes for one component type
pub struct Column {
    pub component_id: u32,
    pub element_size: usize,
    pub data: Vec<u8>,
}

impl Column {
    pub fn new(component_id: u32, element_size: usize) -> Self {
        Self { component_id, element_size, data: Vec::new() }
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.element_size == 0 { 0 } else { self.data.len() / self.element_size }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Read bytes for row `index`
    #[inline]
    pub fn row(&self, index: usize) -> &[u8] {
        let start = index * self.element_size;
        &self.data[start..start + self.element_size]
    }

    /// Read mutable bytes for row `index`
    #[inline]
    pub fn row_mut(&mut self, index: usize) -> &mut [u8] {
        let start = index * self.element_size;
        let end = start + self.element_size;
        &mut self.data[start..end]
    }

    /// Append raw bytes for one row
    #[inline]
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        debug_assert_eq!(bytes.len(), self.element_size);
        self.data.extend_from_slice(bytes);
    }

    /// Swap-remove: copy last row → index, truncate. Returns true if a swap occurred.
    pub fn swap_remove(&mut self, index: usize) {
        let count = self.len();
        debug_assert!(index < count, "swap_remove index out of bounds");
        if index < count - 1 {
            // Copy last element into slot `index`
            let last_start = (count - 1) * self.element_size;
            let dst_start = index * self.element_size;
            self.data.copy_within(last_start..last_start + self.element_size, dst_start);
        }
        let new_len = (count - 1) * self.element_size;
        self.data.truncate(new_len);
    }
}

/// A chunk holds up to CHUNK_CAPACITY entities for one archetype
pub struct Chunk {
    pub entities: Vec<Entity>,
    pub columns: Vec<Column>,
}

impl Chunk {
    pub fn new(template_columns: &[Column]) -> Self {
        let columns = template_columns
            .iter()
            .map(|c| Column::new(c.component_id, c.element_size))
            .collect();
        Self { entities: Vec::new(), columns }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.entities.len() >= CHUNK_CAPACITY
    }

    /// Append one entity row (columns data must be written separately)
    pub fn push_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// Swap-remove entity and all column data. Returns the entity that was moved (if any).
    pub fn swap_remove_row(&mut self, row: usize) -> Option<Entity> {
        let count = self.entities.len();
        debug_assert!(row < count);

        for col in &mut self.columns {
            col.swap_remove(row);
        }

        if row < count - 1 {
            let moved = *self.entities.last().unwrap();
            self.entities.swap_remove(row);
            Some(moved)
        } else {
            self.entities.pop();
            None
        }
    }
}

/// Archetype: all entities with exactly the same component set
pub struct Archetype {
    pub id: u32,
    pub component_ids: Vec<u32>,  // sorted
    pub chunks: Vec<Chunk>,
    pub column_lookup: HashMap<u32, usize>,
    pub mask: ComponentMask,
    /// Template columns (zero-length) for spawning new chunks
    template_columns: Vec<Column>,
}

impl Archetype {
    pub fn new(id: u32, component_ids: Vec<u32>, element_sizes: Vec<usize>, mask: ComponentMask) -> Self {
        let mut lookup = HashMap::new();
        let mut template_columns = Vec::with_capacity(component_ids.len());

        for (i, (&cid, &size)) in component_ids.iter().zip(element_sizes.iter()).enumerate() {
            lookup.insert(cid, i);
            template_columns.push(Column::new(cid, size));
        }

        Self {
            id,
            component_ids,
            chunks: Vec::new(),
            column_lookup: lookup,
            mask,
            template_columns,
        }
    }

    /// Get or create a chunk with space
    pub fn get_or_create_chunk(&mut self) -> usize {
        if self.chunks.is_empty() || self.chunks.last().unwrap().is_full() {
            self.chunks.push(Chunk::new(&self.template_columns));
        }
        self.chunks.len() - 1
    }

    /// Allocate one row — returns (chunk_index, row)
    pub fn allocate_row(&mut self) -> (u32, u32) {
        let chunk_index = self.get_or_create_chunk();
        let row = self.chunks[chunk_index].len();
        (chunk_index as u32, row as u32)
    }

    #[inline]
    pub fn column_index(&self, component_id: u32) -> usize {
        *self.column_lookup.get(&component_id).unwrap()
    }

    pub fn has_component(&self, component_id: u32) -> bool {
        self.column_lookup.contains_key(&component_id)
    }

    pub fn total_entities(&self) -> usize {
        self.chunks.iter().map(|c| c.len()).sum()
    }
}
