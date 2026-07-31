use ecs_core::*;

#[test]
fn send_and_drain_events() {
    let mut world = World::new();
    let damage_id = world.register_event(
        ComponentBuilder::new("DamageEvent")
            .field("target_id", FieldType::U32)
            .field("amount", FieldType::F32)
            .build(),
    );

    world.send_event(damage_id, &[ComponentValue::U32(5), ComponentValue::F32(12.5)]);
    world.send_event(damage_id, &[ComponentValue::U32(6), ComponentValue::F32(3.0)]);

    let events = world.drain_events(damage_id);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0][0].as_u32(), Some(5));
    assert_eq!(events[0][1].as_f32(), Some(12.5));
    assert_eq!(events[1][0].as_u32(), Some(6));
}

#[test]
fn drain_clears_the_queue() {
    let mut world = World::new();
    let id = world.register_event(ComponentBuilder::new("SpawnEvent").field("entity_id", FieldType::U32).build());
    world.send_event(id, &[ComponentValue::U32(1)]);

    assert_eq!(world.drain_events(id).len(), 1);
    assert_eq!(world.drain_events(id).len(), 0, "second drain in the same frame must be empty");
}

#[test]
fn read_does_not_clear_the_queue() {
    let mut world = World::new();
    let id = world.register_event(ComponentBuilder::new("SoundEvent").field("sound_id", FieldType::U32).build());
    world.send_event(id, &[ComponentValue::U32(9)]);

    assert_eq!(world.read_events(id).len(), 1);
    assert_eq!(world.read_events(id).len(), 1, "read() must be non-destructive");
    assert_eq!(world.drain_events(id).len(), 1, "the event is still there for the final drain");
}

#[test]
fn events_fired_earlier_in_the_frame_are_visible_to_later_systems() {
    // Simulates DamageSystem firing DeathEvent, then DeathSystem (a later
    // system in the same frame) reading it — no double-buffering needed
    // because execution is strictly sequential.
    let mut world = World::new();
    let death_id = world.register_event(ComponentBuilder::new("DeathEvent").field("entity_id", FieldType::U32).build());

    // "DamageSystem" stage:
    world.send_event(death_id, &[ComponentValue::U32(42)]);

    // "LootSystem" stage, same frame, runs later:
    let seen = world.read_events(death_id);
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0][0].as_u32(), Some(42));
}

#[test]
fn sending_to_unregistered_event_is_a_silent_no_op() {
    let mut world = World::new();
    world.send_event(999, &[ComponentValue::U32(1)]); // must not panic
    assert_eq!(world.drain_events(999).len(), 0);
}
