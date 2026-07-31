use ecs_core::*;

#[test]
fn query_column() {
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

    let columns = world.query_column(position);

    assert_eq!(columns.len(), 1);

    assert_eq!(columns[0].len(), 8);
}
