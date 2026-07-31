use ecs_core::*;

#[test]
fn read_position_bytes() {
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

    let bytes = world.get_component_bytes(entity, position).unwrap();

    assert_eq!(bytes.len(), 8);
}
