use crate::{ComponentRegistry, ComponentSchema, ComponentValue};

/// Frame-scoped event queue (DamageEvent, DeathEvent, SoundEvent, ...).
///
/// Like [`crate::Resources`], an event type is just a `ComponentSchema` —
/// the payload shape — but instead of one stored instance, each `send`
/// appends a new instance to that event type's queue.
///
/// # Lifetime of an event within a frame
/// Execution in this engine is strictly sequential (no system parallelism
/// yet), so there is no need for Bevy-style double buffering: events queued
/// by an earlier system this frame are immediately visible to `read()` calls
/// made by later systems in the same frame. The host (JS) calls `drain()`
/// once per frame *after* the schedule has finished running, to both read
/// and clear each event type for that frame's Audio/Animation/UI consumers.
pub struct Events {
    registry: ComponentRegistry,
    queues: hashbrown::HashMap<u32, Vec<Vec<u8>>>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            queues: hashbrown::HashMap::new(),
        }
    }

    pub fn register(&mut self, schema: ComponentSchema) -> u32 {
        let id = self.registry.register_schema(schema);
        self.queues.entry(id).or_insert_with(Vec::new);
        id
    }

    pub fn event_id(&self, name: &str) -> Option<u32> {
        self.registry.get_by_name(name).map(|s| s.id)
    }

    pub fn schema(&self, event_id: u32) -> Option<&ComponentSchema> {
        self.registry.get(event_id)
    }

    /// Queue one event instance. No-op (silently dropped) if `event_id`
    /// was never registered — mirrors the forgiving-default behavior the
    /// WASM layer already uses elsewhere (e.g. `get_f32` returning 0.0 for
    /// a dead entity) rather than panicking deep inside a hot system.
    pub fn send(&mut self, event_id: u32, values: &[ComponentValue]) {
        if let Some(schema) = self.registry.get(event_id) {
            let bytes = schema.serialize(values);
            self.queues.entry(event_id).or_insert_with(Vec::new).push(bytes);
        }
    }

    /// Peek at this frame's events so far, without clearing. Use this when
    /// a later system in the same frame needs to react to an event fired
    /// by an earlier system (e.g. LootSystem reading DeathEvent).
    pub fn read(&self, event_id: u32) -> Vec<Vec<ComponentValue>> {
        let schema = match self.registry.get(event_id) {
            Some(s) => s,
            None => return Vec::new(),
        };
        self.queues
            .get(&event_id)
            .map(|q| q.iter().map(|b| schema.deserialize(b)).collect())
            .unwrap_or_default()
    }

    /// Read and clear. Call once per event type, once per frame, after the
    /// schedule has finished running.
    pub fn drain(&mut self, event_id: u32) -> Vec<Vec<ComponentValue>> {
        let schema = match self.registry.get(event_id) {
            Some(s) => s,
            None => return Vec::new(),
        };
        let drained = self
            .queues
            .get_mut(&event_id)
            .map(std::mem::take)
            .unwrap_or_default();
        drained.iter().map(|b| schema.deserialize(b)).collect()
    }

    pub fn clear(&mut self, event_id: u32) {
        if let Some(q) = self.queues.get_mut(&event_id) {
            q.clear();
        }
    }

    pub fn clear_all(&mut self) {
        for q in self.queues.values_mut() {
            q.clear();
        }
    }

    pub fn count(&self, event_id: u32) -> usize {
        self.queues.get(&event_id).map(|q| q.len()).unwrap_or(0)
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}
