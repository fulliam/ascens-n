use ecs_core::*;

#[test]
fn spawn_batch_correct_count() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    let entities = world.spawn_batch(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)]),
    ], 1000);

    assert_eq!(entities.len(), 1000);
    assert_eq!(world.entity_count(), 1000);

    let result = world.query(&[pos]);
    assert_eq!(result.total_rows(), 1000);
}

#[test]
fn spawn_batch_all_alive() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    let entities = world.spawn_batch(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)]),
    ], 100);

    for e in &entities {
        assert!(world.is_alive(*e));
    }
}

#[test]
fn spawn_large_batch_performance() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let vel = world.register_component(
        ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    // Spawn 100k entities  
    world.spawn_batch(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)]),
        ComponentInstance::new(vel, vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)]),
    ], 100_000);

    assert_eq!(world.entity_count(), 100_000);
    let result = world.query(&[pos, vel]);
    assert_eq!(result.total_rows(), 100_000);
}
