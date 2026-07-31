use ecs_core::*;

#[test]
fn access_rows() {
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

    let query = world.query(&[position]);

    let chunk = &query.chunks[0];

    let row = chunk.columns[0].row(0);

    let x = f32::from_le_bytes(row[0..4].try_into().unwrap());

    let y = f32::from_le_bytes(row[4..8].try_into().unwrap());

    assert_eq!(x, 10.0);
    assert_eq!(y, 20.0);
}
