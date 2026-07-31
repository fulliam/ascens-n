use ecs_core::*;

#[test]
fn despawn_entity_with_no_components_does_not_panic() {
    let mut world = World::new();
    let e = world.spawn_empty();
    assert!(world.is_alive(e));
    world.despawn(e).expect("despawning a componentless entity must succeed");
    assert!(!world.is_alive(e));
}

#[test]
fn despawn_entity_with_no_components_then_spawn_more_still_works() {
    let mut world = World::new();
    let e1 = world.spawn_empty();
    world.despawn(e1).unwrap();

    let pos_id = world.register_component(ComponentBuilder::new("Position").field("x", FieldType::F32).build());
    let e2 = world.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(1.0)])]);
    assert!(world.is_alive(e2));
    assert_eq!(world.get_component(e2, pos_id).unwrap()[0].as_f32(), Some(1.0));
}
