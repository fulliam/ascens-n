use ecs_core::*;

#[test]
fn position_is_written() {
    let mut world = World::new();

    let position = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );

    let entity = world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(10.0), ComponentValue::F32(20.0)],
    )]);

    assert!(world.is_alive(entity));

    assert_eq!(world.archetypes[0].chunks[0].entities.len(), 1);

    assert_eq!(world.archetypes[0].chunks[0].columns[0].data.len(), 8);
}
