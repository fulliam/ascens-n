use crate::{
    Archetype, ArchetypeRegistry, ColumnView, ComponentInstance, ComponentMask, ComponentRegistry,
    ComponentSchema, ComponentValue, EcsError, EcsResult, Entity, EntityLocation, EntityManager,
    Events, F32x2View, F32x2ViewMut, F32x3View, QueryChunk, QueryResult, QueryRow, QueryRowMut,
    Resources, Schedule,
};

/// The central ECS container
///
/// # Design
/// - Archetypes for O(1) component-set lookup
/// - Chunk-based SoA storage (cache-line friendly)
/// - Generational entity IDs for O(1) validity
/// - swap-remove for O(1) despawn / component migration
pub struct World {
    pub entity_manager: EntityManager,
    pub archetypes: Vec<Archetype>,
    /// entity_id → location in archetype storage
    pub locations: Vec<EntityLocation>,
    pub component_registry: ComponentRegistry,
    pub archetype_registry: ArchetypeRegistry,
    /// Singleton, non-entity data (Time, Input, Camera, GameState, ...).
    pub resources: Resources,
    /// Frame-scoped event queues (DamageEvent, DeathEvent, ...).
    pub events: Events,
    /// Game systems, grouped into the six standard stages. Empty until
    /// Stage 3 of the roadmap registers real systems via `add_system_to`.
    pub schedule: Schedule,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_manager: EntityManager::new(),
            archetypes: Vec::new(),
            locations: Vec::new(),
            component_registry: ComponentRegistry::new(),
            archetype_registry: ArchetypeRegistry::new(),
            resources: Resources::new(),
            events: Events::new(),
            schedule: Schedule::standard_game_schedule(),
        }
    }

    // ─────────────────────────────────────────────
    // Registration
    // ─────────────────────────────────────────────

    pub fn register_component(&mut self, schema: ComponentSchema) -> u32 {
        self.component_registry.register_schema(schema)
    }

    // ─────────────────────────────────────────────
    // Archetypes
    // ─────────────────────────────────────────────

    pub fn create_archetype(&mut self, mut component_ids: Vec<u32>) -> u32 {
        component_ids.sort_unstable();

        if let Some(id) = self.archetype_registry.get(&component_ids) {
            return id;
        }

        let mut mask = ComponentMask::new();
        let mut element_sizes = Vec::with_capacity(component_ids.len());

        for &cid in &component_ids {
            let schema = self
                .component_registry
                .get(cid)
                .unwrap_or_else(|| panic!("component {} is not registered", cid));
            element_sizes.push(schema.size);
            mask.set(cid as usize);
        }

        let archetype_id = self.archetypes.len() as u32;
        let archetype = Archetype::new(archetype_id, component_ids.clone(), element_sizes, mask);
        self.archetypes.push(archetype);
        self.archetype_registry.insert(component_ids, archetype_id);
        archetype_id
    }

    pub fn get_archetype_id(&self, component_ids: &[u32]) -> Option<u32> {
        let mut sorted = component_ids.to_vec();
        sorted.sort_unstable();
        self.archetype_registry.get(&sorted)
    }

    // ─────────────────────────────────────────────
    // Entity lifecycle
    // ─────────────────────────────────────────────

    pub fn spawn_empty(&mut self) -> Entity {
        let entity = self.entity_manager.create();
        let index = entity.id as usize;
        if self.locations.len() <= index {
            self.locations.resize(index + 1, EntityLocation::INVALID);
        }
        entity
    }

    pub fn spawn(&mut self, components: Vec<ComponentInstance>) -> Entity {
        let entity = self.spawn_empty();

        // Build sorted component_ids
        let mut component_ids: Vec<u32> = components.iter().map(|c| c.component_id).collect();
        component_ids.sort_unstable();

        let archetype_id = self.create_archetype(component_ids.clone());

        // Serialize all components
        let serialized: Vec<(u32, Vec<u8>)> = components
            .iter()
            .map(|c| {
                let schema = self.component_registry.get(c.component_id).unwrap();
                (c.component_id, schema.serialize(&c.values))
            })
            .collect();

        let archetype = &mut self.archetypes[archetype_id as usize];
        let (chunk_index, row) = archetype.allocate_row();
        let chunk = &mut archetype.chunks[chunk_index as usize];

        chunk.push_entity(entity);

        for (component_id, bytes) in &serialized {
            let col_idx = archetype.column_lookup[component_id];
            chunk.columns[col_idx].push_bytes(bytes);
        }
        drop(serialized);

        self.locations[entity.id as usize] = EntityLocation {
            archetype_id,
            chunk_index,
            row,
        };
        entity
    }

    /// Despawn entity (swap-remove, O(1)).
    ///
    /// Handles an entity that was never given any component (`spawn_empty`
    /// with nothing ever added — its location is `EntityLocation::INVALID`
    /// since it was never placed in any archetype) by skipping the
    /// archetype/chunk removal step entirely; there's nothing there to
    /// remove. Previously this asserted `loc.is_valid()` unconditionally,
    /// which panicked in debug builds and would have indexed `archetypes`
    /// with a garbage id in release builds — a real bug, not a hypothetical
    /// one: `CommandBuffer::spawn()` + an immediate `despawn()` before ever
    /// inserting a component hits this exact path.
    pub fn despawn(&mut self, entity: Entity) -> EcsResult<()> {
        if !self.entity_manager.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }

        let loc = self.locations[entity.id as usize];
        if loc.is_valid() {
            let archetype = &mut self.archetypes[loc.archetype_id as usize];
            let chunk = &mut archetype.chunks[loc.chunk_index as usize];

            // swap-remove returns the entity that was moved into `row` position
            if let Some(moved_entity) = chunk.swap_remove_row(loc.row as usize) {
                // Update the moved entity's location
                self.locations[moved_entity.id as usize].row = loc.row;
            }

            self.locations[entity.id as usize] = EntityLocation::INVALID;
        }

        self.entity_manager.destroy(entity);
        Ok(())
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_manager.is_alive(entity)
    }

    pub fn destroy(&mut self, entity: Entity) {
        let _ = self.despawn(entity);
    }

    pub fn alive_count(&self) -> u32 {
        self.entity_manager.alive_count()
    }

    // ─────────────────────────────────────────────
    // Component mutation (archetype migration)
    // ─────────────────────────────────────────────

    /// Add a component to an entity (migrates to new archetype)
    pub fn add_component(&mut self, entity: Entity, component: ComponentInstance) -> EcsResult<()> {
        if !self.entity_manager.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }

        let loc = self.locations[entity.id as usize];

        // Collect current component data.
        // When the entity was spawned empty its location is INVALID (archetype_id = u32::MAX),
        // so we must guard before indexing into the archetypes vec.
        let (mut current_ids, mut migrated) = if loc.is_valid() {
            let ids = self.archetypes[loc.archetype_id as usize]
                .component_ids
                .clone();

            // Already has this component — update in place.
            if ids.contains(&component.component_id) {
                return self.set_component(entity, component);
            }

            let bytes: Vec<(u32, Vec<u8>)> = ids
                .iter()
                .map(|&cid| {
                    let b = self
                        .get_component_bytes_at_location(loc, cid)
                        .unwrap()
                        .to_vec();
                    (cid, b)
                })
                .collect();

            (ids, bytes)
        } else {
            // Entity has no archetype yet (spawned with spawn_empty / spawn_entity).
            // Nothing to migrate; just place it into a fresh archetype below.
            (Vec::new(), Vec::new())
        };

        // Add new component
        let new_schema = self
            .component_registry
            .get(component.component_id)
            .ok_or(EcsError::ComponentNotRegistered(component.component_id))?;
        let new_bytes = new_schema.serialize(&component.values);
        migrated.push((component.component_id, new_bytes));
        current_ids.push(component.component_id);

        // Remove from old archetype
        self.remove_entity_from_archetype(entity, loc);

        // Create new archetype & insert
        let new_arch_id = self.create_archetype(current_ids);
        let archetype = &mut self.archetypes[new_arch_id as usize];
        let (chunk_index, row) = archetype.allocate_row();
        let chunk = &mut archetype.chunks[chunk_index as usize];
        chunk.push_entity(entity);

        for (cid, bytes) in &migrated {
            let col_idx = archetype.column_lookup[cid];
            chunk.columns[col_idx].push_bytes(bytes);
        }

        self.locations[entity.id as usize] = EntityLocation {
            archetype_id: new_arch_id,
            chunk_index,
            row,
        };
        Ok(())
    }

    /// Remove a component from an entity (migrates to new archetype)
    pub fn remove_component(&mut self, entity: Entity, component_id: u32) -> EcsResult<()> {
        if !self.entity_manager.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }

        let loc = self.locations[entity.id as usize];
        let old_arch_id = loc.archetype_id;
        let current_ids = self.archetypes[old_arch_id as usize].component_ids.clone();

        if !current_ids.contains(&component_id) {
            return Err(EcsError::ComponentNotFound(component_id));
        }

        // Gather all bytes except the removed component
        let migrated: Vec<(u32, Vec<u8>)> = current_ids
            .iter()
            .filter(|&&cid| cid != component_id)
            .map(|&cid| {
                let bytes = self
                    .get_component_bytes_at_location(loc, cid)
                    .unwrap()
                    .to_vec();
                (cid, bytes)
            })
            .collect();

        let new_ids: Vec<u32> = current_ids
            .into_iter()
            .filter(|&c| c != component_id)
            .collect();

        // Remove from old archetype
        self.remove_entity_from_archetype(entity, loc);

        if new_ids.is_empty() {
            // entity has no components now — it stays alive but locationless
            self.locations[entity.id as usize] = EntityLocation::INVALID;
            return Ok(());
        }

        // Insert into new archetype
        let new_arch_id = self.create_archetype(new_ids);
        let archetype = &mut self.archetypes[new_arch_id as usize];
        let (chunk_index, row) = archetype.allocate_row();
        let chunk = &mut archetype.chunks[chunk_index as usize];
        chunk.push_entity(entity);

        for (cid, bytes) in &migrated {
            let col_idx = archetype.column_lookup[cid];
            chunk.columns[col_idx].push_bytes(bytes);
        }

        self.locations[entity.id as usize] = EntityLocation {
            archetype_id: new_arch_id,
            chunk_index,
            row,
        };
        Ok(())
    }

    /// Overwrite component data in-place (no migration)
    pub fn set_component(&mut self, entity: Entity, component: ComponentInstance) -> EcsResult<()> {
        if !self.entity_manager.is_alive(entity) {
            return Err(EcsError::EntityNotFound);
        }

        let loc = self.locations[entity.id as usize];
        if !loc.is_valid() {
            return Err(EcsError::ComponentNotFound(component.component_id));
        }

        let schema = self
            .component_registry
            .get(component.component_id)
            .ok_or(EcsError::ComponentNotRegistered(component.component_id))?
            .clone();

        let archetype = &mut self.archetypes[loc.archetype_id as usize];
        if !archetype.has_component(component.component_id) {
            return Err(EcsError::ComponentNotFound(component.component_id));
        }

        let col_idx = archetype.column_index(component.component_id);
        let chunk = &mut archetype.chunks[loc.chunk_index as usize];
        let row_bytes = chunk.columns[col_idx].row_mut(loc.row as usize);

        let new_bytes = schema.serialize(&component.values);
        row_bytes.copy_from_slice(&new_bytes);
        Ok(())
    }

    // ─────────────────────────────────────────────
    // Read access
    // ─────────────────────────────────────────────

    pub fn get_component_bytes(&self, entity: Entity, component_id: u32) -> Option<&[u8]> {
        let loc = self.locations.get(entity.id as usize)?;
        if !loc.is_valid() {
            return None;
        }
        Some(self.get_component_bytes_at_location(*loc, component_id)?)
    }

    fn get_component_bytes_at_location(
        &self,
        loc: EntityLocation,
        component_id: u32,
    ) -> Option<&[u8]> {
        let archetype = self.archetypes.get(loc.archetype_id as usize)?;
        if !archetype.has_component(component_id) {
            return None;
        }
        let col_idx = archetype.column_index(component_id);
        let chunk = archetype.chunks.get(loc.chunk_index as usize)?;
        Some(chunk.columns[col_idx].row(loc.row as usize))
    }

    pub fn get_component(&self, entity: Entity, component_id: u32) -> Option<Vec<ComponentValue>> {
        let bytes = self.get_component_bytes(entity, component_id)?;
        let schema = self.component_registry.get(component_id)?;
        Some(schema.deserialize(bytes))
    }

    pub fn has_component(&self, entity: Entity, component_id: u32) -> bool {
        if !self.entity_manager.is_alive(entity) {
            return false;
        }
        let loc = match self.locations.get(entity.id as usize) {
            Some(l) if l.is_valid() => l,
            _ => return false,
        };
        self.archetypes
            .get(loc.archetype_id as usize)
            .map(|a| a.has_component(component_id))
            .unwrap_or(false)
    }

    pub fn get_f32x2(&self, entity: Entity, component_id: u32) -> Option<F32x2View<'_>> {
        let bytes = self.get_component_bytes(entity, component_id)?;
        Some(F32x2View { data: bytes })
    }

    pub fn get_f32x2_mut(&mut self, entity: Entity, component_id: u32) -> Option<F32x2ViewMut<'_>> {
        if !self.entity_manager.is_alive(entity) {
            return None;
        }
        let loc = *self.locations.get(entity.id as usize)?;
        if !loc.is_valid() {
            return None;
        }
        let archetype = self.archetypes.get_mut(loc.archetype_id as usize)?;
        if !archetype.has_component(component_id) {
            return None;
        }
        let col_idx = archetype.column_index(component_id);
        let chunk = archetype.chunks.get_mut(loc.chunk_index as usize)?;
        let bytes = chunk.columns[col_idx].row_mut(loc.row as usize);
        Some(F32x2ViewMut { data: bytes })
    }

    pub fn get_f32x3(&self, entity: Entity, component_id: u32) -> Option<F32x3View<'_>> {
        let bytes = self.get_component_bytes(entity, component_id)?;
        Some(F32x3View { data: bytes })
    }

    // ─────────────────────────────────────────────
    // Queries
    // ─────────────────────────────────────────────

    /// Query all entities that have ALL of the given component IDs
    pub fn query(&self, component_ids: &[u32]) -> QueryResult<'_> {
        let mut query_mask = ComponentMask::new();
        for &cid in component_ids {
            query_mask.set(cid as usize);
        }

        let mut chunks = Vec::new();
        for archetype in &self.archetypes {
            if !archetype.mask.matches(&query_mask) {
                continue;
            }

            for chunk in &archetype.chunks {
                if chunk.is_empty() {
                    continue;
                }

                let columns: Vec<ColumnView<'_>> = component_ids
                    .iter()
                    .map(|&cid| {
                        let col_idx = archetype.column_index(cid);
                        ColumnView {
                            data: chunk.columns[col_idx].data.as_slice(),
                            element_size: chunk.columns[col_idx].element_size,
                        }
                    })
                    .collect();

                chunks.push(QueryChunk {
                    rows: chunk.len(),
                    columns,
                });
            }
        }
        QueryResult { chunks }
    }

    /// Query returning raw column bytes per archetype chunk
    pub fn query_column(&self, component_id: u32) -> Vec<&[u8]> {
        let mut result = Vec::new();
        for archetype in &self.archetypes {
            if !archetype.mask.contains(component_id as usize) {
                continue;
            }
            let col_idx = archetype.column_index(component_id);
            for chunk in &archetype.chunks {
                result.push(chunk.columns[col_idx].data.as_slice());
            }
        }
        result
    }

    /// Query with exclusion filter
    pub fn query_with_exclude(&self, include: &[u32], exclude: &[u32]) -> QueryResult<'_> {
        let mut include_mask = ComponentMask::new();
        for &cid in include {
            include_mask.set(cid as usize);
        }

        let mut exclude_mask = ComponentMask::new();
        for &cid in exclude {
            exclude_mask.set(cid as usize);
        }

        let mut chunks = Vec::new();
        for archetype in &self.archetypes {
            if !archetype.mask.matches(&include_mask) {
                continue;
            }
            if !archetype.mask.is_disjoint(&exclude_mask) {
                continue;
            }

            for chunk in &archetype.chunks {
                if chunk.is_empty() {
                    continue;
                }
                let columns: Vec<ColumnView<'_>> = include
                    .iter()
                    .map(|&cid| {
                        let col_idx = archetype.column_index(cid);
                        ColumnView {
                            data: chunk.columns[col_idx].data.as_slice(),
                            element_size: chunk.columns[col_idx].element_size,
                        }
                    })
                    .collect();
                chunks.push(QueryChunk {
                    rows: chunk.len(),
                    columns,
                });
            }
        }
        QueryResult { chunks }
    }

    // ─────────────────────────────────────────────
    // Bulk / WASM-optimized
    // ─────────────────────────────────────────────

    /// Spawn N identical entities with the same components (batch-optimized)
    pub fn spawn_batch(&mut self, components: Vec<ComponentInstance>, count: usize) -> Vec<Entity> {
        let mut component_ids: Vec<u32> = components.iter().map(|c| c.component_id).collect();
        component_ids.sort_unstable();

        let archetype_id = self.create_archetype(component_ids.clone());

        // Pre-serialize
        let serialized: Vec<(u32, Vec<u8>)> = components
            .iter()
            .map(|c| {
                let schema = self.component_registry.get(c.component_id).unwrap();
                (c.component_id, schema.serialize(&c.values))
            })
            .collect();

        let mut entities = Vec::with_capacity(count);

        for _ in 0..count {
            let entity = self.entity_manager.create();
            let eid = entity.id as usize;
            if self.locations.len() <= eid {
                self.locations.resize(eid + 1, EntityLocation::INVALID);
            }

            let archetype = &mut self.archetypes[archetype_id as usize];
            let (chunk_index, row) = archetype.allocate_row();
            let chunk = &mut archetype.chunks[chunk_index as usize];
            chunk.push_entity(entity);

            for (cid, bytes) in &serialized {
                let col_idx = archetype.column_lookup[cid];
                chunk.columns[col_idx].push_bytes(bytes);
            }

            self.locations[eid] = EntityLocation {
                archetype_id,
                chunk_index,
                row,
            };
            entities.push(entity);
        }

        entities
    }

    /// Get a flat f32 buffer for a component's first field across all matching chunks
    /// — useful for bulk transfer to JS/WASM consumers
    pub fn get_flat_f32_buffer(&self, component_id: u32) -> Vec<f32> {
        let mut result = Vec::new();
        let schema = match self.component_registry.get(component_id) {
            Some(s) => s,
            None => return result,
        };
        let element_size = schema.size;

        for archetype in &self.archetypes {
            if !archetype.mask.contains(component_id as usize) {
                continue;
            }
            let col_idx = archetype.column_index(component_id);
            for chunk in &archetype.chunks {
                let data = &chunk.columns[col_idx].data;
                // Reinterpret bytes as f32 values (each element)
                for i in 0..chunk.len() {
                    let start = i * element_size;
                    let end = start + 4.min(element_size);
                    if end <= data.len() {
                        result.push(f32::from_le_bytes(
                            data[start..start + 4].try_into().unwrap(),
                        ));
                    }
                }
            }
        }
        result
    }

    /// Write component field values back from a flat f32 buffer (bulk update from JS)
    pub fn set_flat_f32_buffer(&mut self, component_id: u32, field_offset: usize, values: &[f32]) {
        let element_size = match self.component_registry.get(component_id) {
            Some(s) => s.size,
            None => return,
        };

        let mut idx = 0;
        for archetype in &mut self.archetypes {
            if !archetype.mask.contains(component_id as usize) {
                continue;
            }
            let col_idx = archetype.column_lookup[&component_id];
            for chunk in &mut archetype.chunks {
                let data = &mut chunk.columns[col_idx].data;
                for row in 0..chunk.entities.len() {
                    if idx >= values.len() {
                        return;
                    }
                    let dst = row * element_size + field_offset;
                    data[dst..dst + 4].copy_from_slice(&values[idx].to_le_bytes());
                    idx += 1;
                }
            }
        }
    }

    // ─────────────────────────────────────────────
    // Stats
    // ─────────────────────────────────────────────

    pub fn entity_count(&self) -> usize {
        self.entity_manager.alive_count() as usize
    }

    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    pub fn total_entities_in_archetypes(&self) -> usize {
        self.archetypes.iter().map(|a| a.total_entities()).sum()
    }

    // ─────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────

    fn remove_entity_from_archetype(&mut self, _entity: Entity, loc: EntityLocation) {
        if !loc.is_valid() {
            return;
        }

        let archetype = &mut self.archetypes[loc.archetype_id as usize];
        let chunk = &mut archetype.chunks[loc.chunk_index as usize];

        if let Some(moved_entity) = chunk.swap_remove_row(loc.row as usize) {
            self.locations[moved_entity.id as usize].row = loc.row;
        }
    }

    // ─────────────────────────────────────────────
    // Ergonomic game API
    // ─────────────────────────────────────────────

    /// Spawn an empty entity with no components.
    /// Alias for `spawn_empty()`.
    #[inline]
    pub fn spawn_entity(&mut self) -> Entity {
        self.spawn_empty()
    }

    /// Spawn an entity with initial components.
    /// Alias for `spawn()`.
    #[inline]
    pub fn spawn_with(&mut self, components: Vec<ComponentInstance>) -> Entity {
        self.spawn(components)
    }

    /// Returns true if the entity is alive.
    /// Alias for `is_alive()`.
    #[inline]
    pub fn exists(&self, entity: Entity) -> bool {
        self.is_alive(entity)
    }

    /// Add or update a component on an entity.
    /// Migrates to a new archetype if the component is new to this entity.
    /// Alias for `add_component()`.
    #[inline]
    pub fn insert(&mut self, entity: Entity, component: ComponentInstance) -> EcsResult<()> {
        self.add_component(entity, component)
    }

    /// Replace a component value in-place (entity must already have this component).
    /// Alias for `set_component()`.
    #[inline]
    pub fn replace(&mut self, entity: Entity, component: ComponentInstance) -> EcsResult<()> {
        self.set_component(entity, component)
    }

    /// Remove a component from an entity.
    /// Alias for `remove_component()`.
    #[inline]
    pub fn remove(&mut self, entity: Entity, component_id: u32) -> EcsResult<()> {
        self.remove_component(entity, component_id)
    }

    /// Returns true if the entity has the given component.
    /// Alias for `has_component()`.
    #[inline]
    pub fn has(&self, entity: Entity, component_id: u32) -> bool {
        self.has_component(entity, component_id)
    }

    /// Get typed component field values.
    /// Alias for `get_component()`.
    #[inline]
    pub fn get(&self, entity: Entity, component_id: u32) -> Option<Vec<ComponentValue>> {
        self.get_component(entity, component_id)
    }

    /// Look up a component ID by its registered name.
    ///
    /// ```rust,ignore
    /// let pos_id = world.component_id("Position").unwrap();
    /// ```
    #[inline]
    pub fn component_id(&self, name: &str) -> Option<u32> {
        self.component_registry.get_by_name(name).map(|s| s.id)
    }

    /// Iterate all entities with all of the given components (read-only).
    ///
    /// The callback receives a [`QueryRow`] with typed read helpers.
    /// Column indices correspond to the order of `component_ids`.
    ///
    /// This avoids allocating an intermediate `QueryResult` and iterates
    /// archetype storage directly.
    ///
    /// # Example
    /// ```rust,ignore
    /// world.query_each(&[pos_id, vel_id], |row| {
    ///     let px = row.read_f32(0, 0); // Position.x  (col 0, byte offset 0)
    ///     let py = row.read_f32(0, 4); // Position.y  (col 0, byte offset 4)
    ///     let vx = row.read_f32(1, 0); // Velocity.x  (col 1, byte offset 0)
    /// });
    /// ```
    pub fn query_each<F>(&self, component_ids: &[u32], mut f: F)
    where
        F: for<'r> FnMut(QueryRow<'r>),
    {
        let mut query_mask = ComponentMask::new();
        for &cid in component_ids {
            query_mask.set(cid as usize);
        }

        for archetype in &self.archetypes {
            if !archetype.mask.matches(&query_mask) {
                continue;
            }

            // Compute column mapping before iterating chunks
            let col_indices: Vec<usize> = component_ids
                .iter()
                .map(|&cid| archetype.column_index(cid))
                .collect();

            for chunk in &archetype.chunks {
                if chunk.is_empty() {
                    continue;
                }
                let count = chunk.entities.len();
                for row in 0..count {
                    f(QueryRow {
                        columns: &chunk.columns,
                        col_indices: &col_indices,
                        entity: chunk.entities[row],
                        row,
                    });
                }
            }
        }
    }

    /// Iterate all entities with all of the given components with mutable access.
    ///
    /// The callback receives a [`QueryRowMut`] with typed read and write helpers.
    /// All mutations happen in-place — no archetype migration, no allocation.
    ///
    /// # Example — velocity integration
    /// ```rust,ignore
    /// world.query_each_mut(&[pos_id, vel_id], |mut row| {
    ///     let vx = row.read_f32(1, 0); // Velocity.x
    ///     let vy = row.read_f32(1, 4); // Velocity.y
    ///     let px = row.read_f32(0, 0); // Position.x
    ///     let py = row.read_f32(0, 4); // Position.y
    ///     row.write_f32(0, 0, px + vx);
    ///     row.write_f32(0, 4, py + vy);
    /// });
    /// ```
    pub fn query_each_mut<F>(&mut self, component_ids: &[u32], mut f: F)
    where
        F: for<'r> FnMut(QueryRowMut<'r>),
    {
        let mut query_mask = ComponentMask::new();
        for &cid in component_ids {
            query_mask.set(cid as usize);
        }

        for archetype in &mut self.archetypes {
            if !archetype.mask.matches(&query_mask) {
                continue;
            }

            // Build the column index map from the immutable part of archetype
            // before we take the mutable borrow of chunks.
            let col_indices: Vec<usize> = component_ids
                .iter()
                .map(|&cid| *archetype.column_lookup.get(&cid).unwrap())
                .collect();

            for chunk in &mut archetype.chunks {
                let count = chunk.entities.len();
                if count == 0 {
                    continue;
                }
                for row in 0..count {
                    f(QueryRowMut {
                        entity: chunk.entities[row],
                        columns: &mut chunk.columns,
                        col_indices: &col_indices,
                        row,
                    });
                }
            }
        }
    }
    // ─────────────────────────────────────────────
    // Resources (singleton, non-entity data)
    // ─────────────────────────────────────────────

    pub fn register_resource(&mut self, schema: ComponentSchema) -> u32 {
        self.resources.register(schema)
    }

    pub fn resource_id(&self, name: &str) -> Option<u32> {
        self.resources.resource_id(name)
    }

    pub fn has_resource(&self, resource_id: u32) -> bool {
        self.resources.has(resource_id)
    }

    pub fn set_resource(&mut self, resource_id: u32, values: &[ComponentValue]) -> EcsResult<()> {
        self.resources.set(resource_id, values)
    }

    pub fn get_resource(&self, resource_id: u32) -> Option<Vec<ComponentValue>> {
        self.resources.get(resource_id)
    }

    // ─────────────────────────────────────────────
    // Events (frame-scoped queues)
    // ─────────────────────────────────────────────

    pub fn register_event(&mut self, schema: ComponentSchema) -> u32 {
        self.events.register(schema)
    }

    pub fn event_id(&self, name: &str) -> Option<u32> {
        self.events.event_id(name)
    }

    pub fn send_event(&mut self, event_id: u32, values: &[ComponentValue]) {
        self.events.send(event_id, values)
    }

    pub fn read_events(&self, event_id: u32) -> Vec<Vec<ComponentValue>> {
        self.events.read(event_id)
    }

    pub fn drain_events(&mut self, event_id: u32) -> Vec<Vec<ComponentValue>> {
        self.events.drain(event_id)
    }

    // ─────────────────────────────────────────────
    // Schedule
    // ─────────────────────────────────────────────

    /// Run every stage once, in order — one full simulation tick.
    ///
    /// Implementation note: `self.schedule` is temporarily moved out via
    /// `mem::take` so it can be run with `&mut self` (= `world`) passed
    /// back in without a double-borrow of `self`. `Schedule`'s `Default`
    /// is an empty schedule, so this is always cheap and always restored
    /// before the method returns.
    pub fn run_schedule(&mut self) {
        let schedule = std::mem::take(&mut self.schedule);
        schedule.run(self);
        self.schedule = schedule;
    }

    /// Run only the named stage (e.g. called once per stage by the host
    /// loop: `run_stage("Update")`, then `run_stage("Render")`, ...).
    pub fn run_stage(&mut self, stage_name: &str) {
        let schedule = std::mem::take(&mut self.schedule);
        schedule.run_stage(stage_name, self);
        self.schedule = schedule;
    }

    pub fn stage_names(&self) -> Vec<&str> {
        self.schedule.stage_names()
    }

    // ─────────────────────────────────────────────
    // Query identity (Stage 3) — the logic behind JsWorld::query_entities/
    // query_first/query_count. Lives here, not in wasm_api.rs, so it's
    // plain Rust and unit-testable without the `wasm` feature or a real JS
    // environment — `Uint32Array::from(&[u32])` (the one js-sys call this
    // needs) only works inside an actual wasm32 runtime, so wasm_api.rs's
    // wrappers stay a single conversion line each, with everything that
    // can go wrong already covered here.
    // ─────────────────────────────────────────────

    /// (entity_id, generation) for every entity matching ALL given
    /// components, in archetype/chunk iteration order.
    pub fn query_entity_ids(&self, component_ids: &[u32]) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        self.query_each(component_ids, |row| {
            out.push((row.entity.id, row.entity.generation));
        });
        out
    }

    /// The first entity matching ALL given components, or `None`.
    pub fn query_first_id(&self, component_ids: &[u32]) -> Option<(u32, u32)> {
        let mut found = None;
        self.query_each(component_ids, |row| {
            if found.is_none() {
                found = Some((row.entity.id, row.entity.generation));
            }
        });
        found
    }

    /// Count of entities matching ALL given components, without
    /// allocating an id list.
    pub fn query_match_count(&self, component_ids: &[u32]) -> usize {
        let mut count = 0;
        self.query_each(component_ids, |_row| count += 1);
        count
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
