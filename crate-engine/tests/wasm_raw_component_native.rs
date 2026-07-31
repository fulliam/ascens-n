#![cfg(feature = "wasm")]
use ecs_core::wasm_api::JsWorld;

#[test]
fn add_component_raw_round_trip_entity_ref() {
    let mut world = JsWorld::new();
    // EntityRef-shaped relation: id:u32, generation:u32 — exactly what
    // Transform's Parent / Projectile's Owner need.
    let parent_id = world.register_schema("Parent", "id:u32,generation:u32");

    let parent_entity = world.spawn_empty_entity();
    let parent_gen = world.entity_generation(parent_entity);
    let child = world.spawn_empty_entity();
    let child_gen = world.entity_generation(child);

    let mut bytes = Vec::with_capacity(8);
    bytes.extend_from_slice(&parent_entity.to_le_bytes());
    bytes.extend_from_slice(&parent_gen.to_le_bytes());

    assert!(world.add_component_raw(child, child_gen, parent_id, &bytes), "add_component_raw succeeds for a 2x u32 schema");

    let read_back = world.get_component_raw(child, child_gen, parent_id);
    assert_eq!(read_back, bytes, "get_component_raw returns exactly what was written");

    let read_id = u32::from_le_bytes(read_back[0..4].try_into().unwrap());
    let read_gen = u32::from_le_bytes(read_back[4..8].try_into().unwrap());
    assert_eq!(read_id, parent_entity);
    assert_eq!(read_gen, parent_gen);

    // overwrite via set_component_raw with a different parent
    let other_parent = world.spawn_empty_entity();
    let other_gen = world.entity_generation(other_parent);
    let mut other_bytes = Vec::with_capacity(8);
    other_bytes.extend_from_slice(&other_parent.to_le_bytes());
    other_bytes.extend_from_slice(&other_gen.to_le_bytes());
    assert!(world.set_component_raw(child, child_gen, parent_id, &other_bytes));
    assert_eq!(world.get_component_raw(child, child_gen, parent_id), other_bytes);
}

#[test]
fn add_component_raw_rejects_wrong_length() {
    let mut world = JsWorld::new();
    let id = world.register_schema("Owner", "id:u32,generation:u32"); // 8 bytes
    let e = world.spawn_empty_entity();
    let gen = world.entity_generation(e);
    assert!(!world.add_component_raw(e, gen, id, &[0u8; 4]), "wrong byte length is rejected, not silently truncated");
    assert!(!world.has_component(e, gen, id));
}

#[test]
fn raw_accessors_are_lenient_on_dead_entity_or_unregistered_component() {
    let mut world = JsWorld::new();
    let id = world.register_schema("X", "a:u32");
    let e = world.spawn_empty_entity();
    let gen = world.entity_generation(e);
    world.despawn(e, gen);

    assert!(!world.add_component_raw(e, gen, id, &[0u8; 4]));
    assert_eq!(world.get_component_raw(e, gen, id), Vec::<u8>::new());
    assert!(!world.set_component_raw(e, gen, id, &[0u8; 4]));
    assert_eq!(world.get_component_raw(0, 0, 999_999), Vec::<u8>::new(), "unregistered component id returns empty, not a panic");
}
