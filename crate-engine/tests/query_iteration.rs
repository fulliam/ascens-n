use ecs_core::*;

#[test]
fn iterate_position_column() {
    let mut world = World::new();

    let position = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );

    world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(10.0), ComponentValue::F32(20.0)],
    )]);

    world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(30.0), ComponentValue::F32(40.0)],
    )]);

    let query = world.query(&[position]);

    let chunk = &query.chunks[0];

    let column = &chunk.columns[0];

    assert_eq!(column.len(), 2);
}
