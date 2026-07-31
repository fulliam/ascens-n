use crate::{ComponentInstance, Entity, World};

/// A single deferred structural mutation.
enum Command {
    Spawn(Vec<ComponentInstance>),
    Despawn(Entity),
    AddComponent(Entity, ComponentInstance),
    RemoveComponent(Entity, u32),
    SetComponent(Entity, ComponentInstance),
}

/// Deferred mutation buffer for use *inside* `World::query_each` /
/// `query_each_mut` closures, where a live `&mut World` borrow already
/// belongs to the query and isn't available.
///
/// `CommandBuffer` is intentionally **not** a field on `World` — it must be
/// a plain local value, independent of `world`, so it can be captured by a
/// query closure without conflicting with the query's own borrow of
/// `world`. The pattern used by every system that needs this:
///
/// ```rust,ignore
/// fn run(&self, world: &mut World) {
///     let mut commands = CommandBuffer::new();
///
///     world.query_each(&[health_id], |row| {
///         if row.read_f32(0, 0) <= 0.0 {
///             // can't touch `world` here — only `commands`, a separate local
///             commands.despawn(row.entity);
///         }
///     });
///
///     // `world` is free again now that the query has returned.
///     commands.apply(world);
/// }
/// ```
///
/// This is the same "collect intents, then apply after the loop" shape the
/// existing `step_physics` already uses internally — `CommandBuffer` just
/// gives every game system the same pattern as a reusable, named type
/// instead of an ad-hoc `Vec` per system.
#[derive(Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, components: Vec<ComponentInstance>) {
        self.commands.push(Command::Spawn(components));
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.commands.push(Command::Despawn(entity));
    }

    pub fn add_component(&mut self, entity: Entity, component: ComponentInstance) {
        self.commands.push(Command::AddComponent(entity, component));
    }

    pub fn remove_component(&mut self, entity: Entity, component_id: u32) {
        self.commands.push(Command::RemoveComponent(entity, component_id));
    }

    pub fn set_component(&mut self, entity: Entity, component: ComponentInstance) {
        self.commands.push(Command::SetComponent(entity, component));
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Apply every recorded command to `world`, in recording order, then
    /// clear the buffer (so the same `CommandBuffer` can be reused next
    /// frame without reallocating).
    pub fn apply(&mut self, world: &mut World) {
        for command in self.commands.drain(..) {
            match command {
                Command::Spawn(components) => {
                    world.spawn(components);
                }
                Command::Despawn(entity) => {
                    let _ = world.despawn(entity);
                }
                Command::AddComponent(entity, component) => {
                    let _ = world.add_component(entity, component);
                }
                Command::RemoveComponent(entity, component_id) => {
                    let _ = world.remove_component(entity, component_id);
                }
                Command::SetComponent(entity, component) => {
                    let _ = world.set_component(entity, component);
                }
            }
        }
    }
}
