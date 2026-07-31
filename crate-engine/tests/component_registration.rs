use ecs_core::*;

#[test]
fn register_position() {
    let mut world = World::new();

    let id = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );

    assert_eq!(id, 0);

    let schema = world.component_registry.get(id).unwrap();

    assert_eq!(schema.size, 8);

    assert_eq!(schema.fields.len(), 2);
}
