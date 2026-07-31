use ecs_core::*;

fn make_world() -> (World, u32, u32) {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let vel = world.register_component(
        ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    (world, pos, vel)
}

#[test]
fn spawn_and_alive() {
    let (mut world, pos, _) = make_world();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    assert!(world.is_alive(e));
    assert_eq!(world.entity_count(), 1);
}

#[test]
fn despawn_entity() {
    let (mut world, pos, _) = make_world();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    assert!(world.is_alive(e));
    world.despawn(e).unwrap();
    assert!(!world.is_alive(e));
    assert_eq!(world.entity_count(), 0);
}

#[test]
fn despawn_dead_entity_returns_error() {
    let (mut world, pos, _) = make_world();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    world.despawn(e).unwrap();
    assert!(matches!(world.despawn(e), Err(EcsError::EntityNotFound)));
}

#[test]
fn entity_id_reused_after_despawn() {
    let (mut world, pos, _) = make_world();
    let e1 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    world.despawn(e1).unwrap();
    let e2 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    // id is reused but generation bumped
    assert_eq!(e1.id, e2.id);
    assert_ne!(e1.generation, e2.generation);
}

#[test]
fn despawn_swap_removes_last_entity() {
    let (mut world, pos, _) = make_world();
    let e1 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    let e2 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)])]);
    let e3 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(5.0), ComponentValue::F32(6.0)])]);

    // Remove middle entity — e3 should be moved into its slot
    world.despawn(e2).unwrap();
    assert!(!world.is_alive(e2));
    assert!(world.is_alive(e1));
    assert!(world.is_alive(e3));

    // e3's data should still be readable
    let vals = world.get_component(e3, pos).unwrap();
    assert_eq!(vals[0].as_f32().unwrap(), 5.0);
    assert_eq!(vals[1].as_f32().unwrap(), 6.0);
}

#[test]
fn alive_count_accurate() {
    let (mut world, pos, _) = make_world();
    assert_eq!(world.entity_count(), 0);
    let e1 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    let e2 = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    assert_eq!(world.entity_count(), 2);
    world.despawn(e1).unwrap();
    assert_eq!(world.entity_count(), 1);
    world.despawn(e2).unwrap();
    assert_eq!(world.entity_count(), 0);
}
