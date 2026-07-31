use ecs_core::*;

#[test]
fn query_position() {
    let mut world = World::new();

    let position = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );

    world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
    )]);

    world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)],
    )]);

    let result = world.query(&[position]);

    assert_eq!(result.chunks.len(), 1);

    assert_eq!(result.chunks[0].rows, 2);
}
