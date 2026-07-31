use ecs_core::*;

#[test]
fn register_and_round_trip_resource() {
    let mut world = World::new();
    let time_id = world.register_resource(
        ComponentBuilder::new("Time")
            .field("elapsed", FieldType::F32)
            .field("delta", FieldType::F32)
            .build(),
    );

    world
        .set_resource(time_id, &[ComponentValue::F32(1.5), ComponentValue::F32(0.016)])
        .unwrap();

    let values = world.get_resource(time_id).unwrap();
    assert_eq!(values[0].as_f32(), Some(1.5));
    assert_eq!(values[1].as_f32(), Some(0.016));
}

#[test]
fn resource_lookup_by_name_is_idempotent() {
    let mut world = World::new();
    let a = world.register_resource(ComponentBuilder::new("GameState").field("paused", FieldType::Bool).build());
    let b = world.register_resource(ComponentBuilder::new("GameState").field("paused", FieldType::Bool).build());
    assert_eq!(a, b);
    assert_eq!(world.resource_id("GameState"), Some(a));
}

#[test]
fn unregistered_resource_is_absent() {
    let world = World::new();
    assert_eq!(world.resource_id("DoesNotExist"), None);
    assert!(!world.has_resource(999));
    assert!(world.get_resource(999).is_none());
}

#[test]
fn setting_unregistered_resource_errors() {
    let mut world = World::new();
    let result = world.set_resource(999, &[ComponentValue::F32(1.0)]);
    assert!(result.is_err());
}

#[test]
fn resources_are_independent_of_components_of_the_same_name() {
    // Resources use their own registry, separate from world.component_registry —
    // a "Time" resource and a "Time" component (if anyone ever registered one)
    // must not collide.
    let mut world = World::new();
    let comp_id = world.register_component(ComponentBuilder::new("Time").field("x", FieldType::F32).build());
    let res_id = world.register_resource(ComponentBuilder::new("Time").field("elapsed", FieldType::F32).build());

    world.set_resource(res_id, &[ComponentValue::F32(42.0)]).unwrap();
    let e = world.spawn(vec![ComponentInstance::new(comp_id, vec![ComponentValue::F32(7.0)])]);

    assert_eq!(world.get_resource(res_id).unwrap()[0].as_f32(), Some(42.0));
    assert_eq!(world.get_component(e, comp_id).unwrap()[0].as_f32(), Some(7.0));
}
