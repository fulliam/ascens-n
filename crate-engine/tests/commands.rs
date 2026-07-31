use ecs_core::*;

#[test]
fn deferred_spawn_via_commands() {
    let mut world = World::new();
    let pos_id = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build(),
    );

    let mut commands = CommandBuffer::new();
    commands.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    assert_eq!(world.entity_count(), 0, "nothing should exist before apply()");

    commands.apply(&mut world);
    assert_eq!(world.entity_count(), 1);
    assert!(commands.is_empty(), "apply() must drain the buffer");
}

#[test]
fn deferred_despawn_found_via_query_each() {
    // The motivating scenario: find dead entities via a read-only query
    // (where `world` is borrowed and unavailable), queue their despawn via
    // CommandBuffer, then apply after the query has released its borrow.
    let mut world = World::new();
    let health_id = world.register_component(ComponentBuilder::new("Health").field("value", FieldType::F32).build());

    let alive = world.spawn(vec![ComponentInstance::new(health_id, vec![ComponentValue::F32(50.0)])]);
    let dead = world.spawn(vec![ComponentInstance::new(health_id, vec![ComponentValue::F32(0.0)])]);

    let mut commands = CommandBuffer::new();
    world.query_each(&[health_id], |row| {
        if row.read_f32(0, 0) <= 0.0 {
            commands.despawn(row.entity);
        }
    });
    commands.apply(&mut world);

    assert!(world.is_alive(alive));
    assert!(!world.is_alive(dead));
}

#[test]
fn deferred_add_remove_set_component() {
    let mut world = World::new();
    let pos_id = world.register_component(ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build());
    let vel_id = world.register_component(ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build());

    let e = world.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);

    let mut commands = CommandBuffer::new();
    commands.add_component(e, ComponentInstance::new(vel_id, vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)]));
    commands.apply(&mut world);
    assert!(world.has_component(e, vel_id));

    let mut commands = CommandBuffer::new();
    commands.set_component(e, ComponentInstance::new(pos_id, vec![ComponentValue::F32(9.0), ComponentValue::F32(9.0)]));
    commands.apply(&mut world);
    assert_eq!(world.get_component(e, pos_id).unwrap()[0].as_f32(), Some(9.0));

    let mut commands = CommandBuffer::new();
    commands.remove_component(e, vel_id);
    commands.apply(&mut world);
    assert!(!world.has_component(e, vel_id));
}

#[test]
fn command_buffer_can_be_reused_after_apply() {
    let mut world = World::new();
    let pos_id = world.register_component(ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build());

    let mut commands = CommandBuffer::new();
    commands.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    commands.apply(&mut world);

    commands.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(1.0), ComponentValue::F32(1.0)])]);
    commands.apply(&mut world);

    assert_eq!(world.entity_count(), 2);
}
