use ecs_core::*;

fn world_with_pos_vel() -> (World, u32, u32) {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let vel = world.register_component(
        ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    (world, pos, vel)
}

#[test]
fn set_component_updates_value() {
    let (mut world, pos, _) = world_with_pos_vel();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);

    world.set_component(e, ComponentInstance::new(pos, vec![ComponentValue::F32(99.0), ComponentValue::F32(88.0)])).unwrap();

    let vals = world.get_component(e, pos).unwrap();
    assert_eq!(vals[0].as_f32().unwrap(), 99.0);
    assert_eq!(vals[1].as_f32().unwrap(), 88.0);
}

#[test]
fn add_component_migrates_archetype() {
    let (mut world, pos, vel) = world_with_pos_vel();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);

    assert!(!world.has_component(e, vel));
    assert_eq!(world.archetype_count(), 1);

    world.add_component(e, ComponentInstance::new(vel, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)])).unwrap();

    assert!(world.has_component(e, pos));
    assert!(world.has_component(e, vel));
    assert_eq!(world.archetype_count(), 2);

    let pos_vals = world.get_component(e, pos).unwrap();
    assert_eq!(pos_vals[0].as_f32().unwrap(), 1.0);

    let vel_vals = world.get_component(e, vel).unwrap();
    assert_eq!(vel_vals[0].as_f32().unwrap(), 3.0);
}

#[test]
fn remove_component_migrates_archetype() {
    let (mut world, pos, vel) = world_with_pos_vel();
    let e = world.spawn(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(10.0), ComponentValue::F32(20.0)]),
        ComponentInstance::new(vel, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)]),
    ]);

    assert!(world.has_component(e, vel));
    world.remove_component(e, vel).unwrap();
    assert!(!world.has_component(e, vel));
    assert!(world.has_component(e, pos));

    let pos_vals = world.get_component(e, pos).unwrap();
    assert_eq!(pos_vals[0].as_f32().unwrap(), 10.0);
}

#[test]
fn remove_missing_component_returns_error() {
    let (mut world, pos, vel) = world_with_pos_vel();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    assert!(matches!(world.remove_component(e, vel), Err(EcsError::ComponentNotFound(_))));
}

#[test]
fn add_component_on_dead_entity_returns_error() {
    let (mut world, pos, vel) = world_with_pos_vel();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    world.despawn(e).unwrap();
    assert!(matches!(
        world.add_component(e, ComponentInstance::new(vel, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])),
        Err(EcsError::EntityNotFound)
    ));
}

#[test]
fn get_f32x2_mut_modifies_in_place() {
    let (mut world, pos, _) = world_with_pos_vel();
    let e = world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(5.0), ComponentValue::F32(6.0)])]);

    {
        let mut view = world.get_f32x2_mut(e, pos).unwrap();
        view.set_x(100.0);
        view.set_y(200.0);
    }

    let view = world.get_f32x2(e, pos).unwrap();
    assert_eq!(view.x(), 100.0);
    assert_eq!(view.y(), 200.0);
}
