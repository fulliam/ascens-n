use ecs_core::*;

#[test]
fn query_entity_ids_matches_all_given_components() {
    let mut world = World::new();
    let pos_id = world.register_component(ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build());
    let vel_id = world.register_component(ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build());

    let e1 = world.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]);
    let _e2 = world.spawn(vec![ComponentInstance::new(pos_id, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)])]); // Position only
    let e3 = world.spawn(vec![
        ComponentInstance::new(pos_id, vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)]),
        ComponentInstance::new(vel_id, vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)]),
    ]);

    let both = world.query_entity_ids(&[pos_id, vel_id]);
    assert_eq!(both, vec![(e3.id, e3.generation)]);

    let pos_only = world.query_entity_ids(&[pos_id]);
    assert_eq!(pos_only.len(), 3);

    let _ = e1;
}

#[test]
fn query_entity_ids_empty_when_nothing_matches() {
    let world = World::new();
    assert_eq!(world.query_entity_ids(&[123]), Vec::new());
}

#[test]
fn query_first_id_returns_none_or_a_real_match() {
    let mut world = World::new();
    let tag_id = world.register_component(ComponentBuilder::new("Tag").field("v", FieldType::U32).build());

    assert_eq!(world.query_first_id(&[tag_id]), None);

    let e1 = world.spawn(vec![ComponentInstance::new(tag_id, vec![ComponentValue::U32(1)])]);
    let found = world.query_first_id(&[tag_id]);
    assert_eq!(found, Some((e1.id, e1.generation)));
}

#[test]
fn query_match_count_does_not_require_collecting_ids() {
    let mut world = World::new();
    let tag_id = world.register_component(ComponentBuilder::new("Tag").field("v", FieldType::U32).build());

    assert_eq!(world.query_match_count(&[tag_id]), 0);

    for _ in 0..5 {
        world.spawn(vec![ComponentInstance::new(tag_id, vec![ComponentValue::U32(1)])]);
    }
    assert_eq!(world.query_match_count(&[tag_id]), 5);
}

#[test]
fn query_ignores_despawned_entities() {
    let mut world = World::new();
    let tag_id = world.register_component(ComponentBuilder::new("Tag").field("v", FieldType::U32).build());
    let e1 = world.spawn(vec![ComponentInstance::new(tag_id, vec![ComponentValue::U32(1)])]);
    let _e2 = world.spawn(vec![ComponentInstance::new(tag_id, vec![ComponentValue::U32(1)])]);

    world.despawn(e1);

    assert_eq!(world.query_match_count(&[tag_id]), 1);
    assert!(!world.query_entity_ids(&[tag_id]).contains(&(e1.id, e1.generation)));
}
