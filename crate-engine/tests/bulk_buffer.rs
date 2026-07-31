use ecs_core::*;

#[test]
fn flat_f32_buffer_round_trip() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)])]);

    // Write [10.0, 20.0] as x-coordinates for both entities  
    let new_xs = vec![10.0f32, 20.0];
    world.set_flat_f32_buffer(pos, 0, &new_xs);

    let xs = world.get_flat_f32_buffer(pos);
    assert_eq!(xs.len(), 2);
    assert_eq!(xs[0], 10.0);
    assert_eq!(xs[1], 20.0);
}

#[test]
fn flat_f32_buffer_large() {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );

    world.spawn_batch(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)]),
    ], 10_000);

    let buf = world.get_flat_f32_buffer(pos);
    assert_eq!(buf.len(), 10_000);
}
