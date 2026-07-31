#![cfg(feature = "wasm")]
use ecs_core::wasm_api::JsWorld;

#[test]
fn get_bool_set_bool_round_trip() {
    let mut world = JsWorld::new();
    let alive_id = world.register_bool("Alive");

    let e1 = world.spawn_empty_entity();
    let gen1 = world.entity_generation(e1);
    let e2 = world.spawn_empty_entity();
    let gen2 = world.entity_generation(e2);

    // Closes the Stage 2 gap: add_component_bool existed, get_bool didn't.
    world.add_component_bool(e1, gen1, alive_id, true);
    world.add_component_bool(e2, gen2, alive_id, false);

    assert!(world.get_bool(e1, gen1, alive_id, 0));
    assert!(!world.get_bool(e2, gen2, alive_id, 0));

    world.set_bool(e2, gen2, alive_id, 0, true);
    assert!(world.get_bool(e2, gen2, alive_id, 0));

    // dead/missing-component reads default to false, same leniency as get_f32/get_u32
    let e3 = world.spawn_empty_entity();
    let gen3 = world.entity_generation(e3);
    assert!(!world.get_bool(e3, gen3, alive_id, 0));
}

#[test]
fn query_count_wasm_wrapper_matches_inner_logic() {
    let mut world = JsWorld::new();
    let tag_id = world.register_u32("Tag");
    for _ in 0..3 {
        world.spawn_u32(tag_id, 1);
    }
    assert_eq!(world.query_count(&[tag_id]), 3);
    assert_eq!(world.query_count(&[999_999]), 0);
}
