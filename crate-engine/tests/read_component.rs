use ecs_core::*;

#[test]
fn read_position() {
    let mut world = World::new();

    let position = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );

    let entity = world.spawn(vec![ComponentInstance::new(
        position,
        vec![ComponentValue::F32(100.0), ComponentValue::F32(200.0)],
    )]);

    let values = world.get_component(entity, position).unwrap();

    match &values[0] {
        ComponentValue::F32(v) => assert_eq!(*v, 100.0),
        _ => panic!(),
    }

    match &values[1] {
        ComponentValue::F32(v) => assert_eq!(*v, 200.0),
        _ => panic!(),
    }
}
