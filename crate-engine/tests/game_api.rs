use ecs_core::*;

// ── Setup helpers ─────────────────────────────────────────────────────────────

fn make_world() -> (World, u32, u32) {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );
    let vel = world.register_component(
        ComponentBuilder::new("Velocity")
            .field("x", FieldType::F32)
            .field("y", FieldType::F32)
            .build(),
    );
    (world, pos, vel)
}

// ── Entity API ────────────────────────────────────────────────────────────────

#[test]
fn spawn_entity_alias() {
    let (mut world, _pos, _) = make_world();
    let e = world.spawn_entity();
    assert!(world.exists(e));
    assert!(world.is_alive(e)); // original still works
}

#[test]
fn spawn_with_alias() {
    let (mut world, pos, vel) = make_world();
    let e = world.spawn_with(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(0.5), ComponentValue::F32(0.0)],
        ),
    ]);
    assert!(world.exists(e));
    assert!(world.has(e, pos));
    assert!(world.has(e, vel));
}

// ── Component API ─────────────────────────────────────────────────────────────

#[test]
fn insert_alias_adds_component() {
    let (mut world, pos, _) = make_world();
    let e = world.spawn_entity();

    world
        .insert(
            e,
            ComponentInstance::new(
                pos,
                vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)],
            ),
        )
        .unwrap();

    assert!(world.has(e, pos));
    let vals = world.get(e, pos).unwrap();
    assert_eq!(vals[0].as_f32().unwrap(), 3.0);
    assert_eq!(vals[1].as_f32().unwrap(), 4.0);
}

#[test]
fn replace_alias_updates_in_place() {
    let (mut world, pos, _) = make_world();
    let e = world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
    )]);

    world
        .replace(
            e,
            ComponentInstance::new(
                pos,
                vec![ComponentValue::F32(99.0), ComponentValue::F32(88.0)],
            ),
        )
        .unwrap();

    let vals = world.get(e, pos).unwrap();
    assert_eq!(vals[0].as_f32().unwrap(), 99.0);
    assert_eq!(vals[1].as_f32().unwrap(), 88.0);
}

#[test]
fn remove_alias_removes_component() {
    let (mut world, pos, vel) = make_world();
    let e = world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(0.1), ComponentValue::F32(0.0)],
        ),
    ]);

    assert!(world.has(e, vel));
    world.remove(e, vel).unwrap();
    assert!(!world.has(e, vel));
    assert!(world.has(e, pos)); // pos still there
}

#[test]
fn has_and_get_aliases() {
    let (mut world, pos, vel) = make_world();
    let e = world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(5.0), ComponentValue::F32(6.0)],
    )]);

    assert!(world.has(e, pos));
    assert!(!world.has(e, vel));

    let vals = world.get(e, pos).unwrap();
    assert_eq!(vals[0].as_f32().unwrap(), 5.0);
}

#[test]
fn component_id_by_name() {
    let (world, pos, _) = make_world();
    assert_eq!(world.component_id("Position"), Some(pos));
    assert_eq!(world.component_id("NonExistent"), None);
}

// ── QueryResult ergonomic API ─────────────────────────────────────────────────

#[test]
fn query_result_iter_alias() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(2.0), ComponentValue::F32(0.0)],
    )]);

    let result = world.query(&[pos]);
    assert_eq!(result.iter().count(), 2);
}

#[test]
fn query_result_single_one_entity() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(42.0), ComponentValue::F32(0.0)],
    )]);

    let result = world.query(&[pos]);
    assert!(result.single().is_some());
}

#[test]
fn query_result_single_multiple_entities_returns_none() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(2.0), ComponentValue::F32(0.0)],
    )]);

    let result = world.query(&[pos]);
    assert!(result.single().is_none());
}

#[test]
fn query_result_single_no_entities_returns_none() {
    let (world, pos, _) = make_world();
    let result = world.query(&[pos]);
    assert!(result.single().is_none());
}

#[test]
fn query_result_first() {
    let (mut world, pos, _) = make_world();
    let result = world.query(&[pos]);
    assert!(result.first().is_none()); // empty

    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)],
    )]);

    let result = world.query(&[pos]);
    assert!(result.first().is_some());
    assert_eq!(result.total_rows(), 2);
}

#[test]
fn query_result_for_each() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(10.0), ComponentValue::F32(0.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(20.0), ComponentValue::F32(0.0)],
    )]);

    let result = world.query(&[pos]);
    let mut count = 0usize;
    result.for_each(|_chunk, _row| count += 1);
    assert_eq!(count, 2);
}

#[test]
fn query_result_for_each_row_reads_values() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)],
    )]);

    let result = world.query(&[pos]);
    let mut xs = Vec::new();
    result.for_each_row(|cols, row| {
        xs.push(cols[0].read_f32_at(row, 0));
    });
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(xs, vec![1.0, 3.0]);
}

// ── World::query_each ─────────────────────────────────────────────────────────

#[test]
fn query_each_single_component() {
    let (mut world, pos, _) = make_world();
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(10.0), ComponentValue::F32(20.0)],
    )]);
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(30.0), ComponentValue::F32(40.0)],
    )]);

    let mut xs = Vec::new();
    world.query_each(&[pos], |row| {
        xs.push(row.read_f32(0, 0)); // Position.x
    });
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(xs, vec![10.0, 30.0]);
}

#[test]
fn query_each_multi_component() {
    let (mut world, pos, vel) = make_world();

    // Two entities with pos+vel
    world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(0.1), ComponentValue::F32(0.2)],
        ),
    ]);
    world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(0.3), ComponentValue::F32(0.4)],
        ),
    ]);
    // One entity with pos only — should NOT appear in the query
    world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(100.0), ComponentValue::F32(0.0)],
    )]);

    let mut count = 0usize;
    let mut px_sum = 0.0f32;
    world.query_each(&[pos, vel], |row| {
        count += 1;
        px_sum += row.read_f32(0, 0);
    });

    assert_eq!(count, 2);
    assert!((px_sum - 4.0).abs() < 0.001); // 1.0 + 3.0
}

#[test]
fn query_each_reads_both_components() {
    let (mut world, pos, vel) = make_world();
    world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(5.0), ComponentValue::F32(6.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(0.5), ComponentValue::F32(0.6)],
        ),
    ]);

    world.query_each(&[pos, vel], |row| {
        assert!((row.read_f32(0, 0) - 5.0).abs() < 0.001); // pos.x
        assert!((row.read_f32(0, 4) - 6.0).abs() < 0.001); // pos.y
        assert!((row.read_f32(1, 0) - 0.5).abs() < 0.001); // vel.x
        assert!((row.read_f32(1, 4) - 0.6).abs() < 0.001); // vel.y
    });
}

// ── World::query_each_mut ─────────────────────────────────────────────────────

#[test]
fn query_each_mut_velocity_integration() {
    let (mut world, pos, vel) = make_world();

    // Spawn entity at (0, 0) with velocity (1, 2)
    let e = world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)],
        ),
    ]);

    // Apply velocity once (dt = 1.0)
    world.query_each_mut(&[pos, vel], |mut row| {
        let vx = row.read_f32(1, 0);
        let vy = row.read_f32(1, 4);
        let px = row.read_f32(0, 0);
        let py = row.read_f32(0, 4);
        row.write_f32(0, 0, px + vx);
        row.write_f32(0, 4, py + vy);
    });

    let vals = world.get(e, pos).unwrap();
    assert!((vals[0].as_f32().unwrap() - 1.0).abs() < 0.001);
    assert!((vals[1].as_f32().unwrap() - 2.0).abs() < 0.001);
}

#[test]
fn query_each_mut_multiple_entities() {
    let (mut world, pos, vel) = make_world();

    let e1 = world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)],
        ),
    ]);
    let e2 = world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(10.0), ComponentValue::F32(0.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(2.0), ComponentValue::F32(0.0)],
        ),
    ]);

    world.query_each_mut(&[pos, vel], |mut row| {
        let vx = row.read_f32(1, 0);
        let px = row.read_f32(0, 0);
        row.write_f32(0, 0, px + vx);
    });

    let v1 = world.get(e1, pos).unwrap();
    let v2 = world.get(e2, pos).unwrap();
    assert!((v1[0].as_f32().unwrap() - 1.0).abs() < 0.001);
    assert!((v2[0].as_f32().unwrap() - 12.0).abs() < 0.001);
}

#[test]
fn query_each_mut_does_not_touch_pos_only_entities() {
    let (mut world, pos, vel) = make_world();

    // pos+vel entity — will be mutated
    let e_moving = world.spawn(vec![
        ComponentInstance::new(
            pos,
            vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)],
        ),
        ComponentInstance::new(
            vel,
            vec![ComponentValue::F32(5.0), ComponentValue::F32(0.0)],
        ),
    ]);
    // pos-only entity — must NOT be touched
    let e_static = world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(99.0), ComponentValue::F32(0.0)],
    )]);

    world.query_each_mut(&[pos, vel], |mut row| {
        let vx = row.read_f32(1, 0);
        let px = row.read_f32(0, 0);
        row.write_f32(0, 0, px + vx);
    });

    let mv = world.get(e_moving, pos).unwrap();
    let sv = world.get(e_static, pos).unwrap();
    assert!((mv[0].as_f32().unwrap() - 5.0).abs() < 0.001);
    assert!((sv[0].as_f32().unwrap() - 99.0).abs() < 0.001); // unchanged
}

#[test]
fn query_each_mut_write_u32() {
    let mut world = World::new();
    let state = world.register_component(
        ComponentBuilder::new("State")
            .field("value", FieldType::U32)
            .build(),
    );

    let e = world.spawn(vec![ComponentInstance::new(
        state,
        vec![ComponentValue::U32(0)],
    )]);

    world.query_each_mut(&[state], |mut row| {
        let current = row.read_u32(0, 0);
        row.write_u32(0, 0, current + 10);
    });

    let vals = world.get(e, state).unwrap();
    assert_eq!(vals[0].as_u32().unwrap(), 10);
}

// ── EntityManager::generation_of ─────────────────────────────────────────────

#[test]
fn entity_generation_increments_on_respawn() {
    let (mut world, pos, _) = make_world();

    let e = world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(0.0), ComponentValue::F32(0.0)],
    )]);

    let gen0 = world.entity_manager.generation_of(e.id);
    assert_eq!(gen0, e.generation);

    world.despawn(e).unwrap();

    // Respawn — reuses the ID but increments generation
    let e2 = world.spawn(vec![ComponentInstance::new(
        pos,
        vec![ComponentValue::F32(1.0), ComponentValue::F32(0.0)],
    )]);

    if e2.id == e.id {
        let gen1 = world.entity_manager.generation_of(e.id);
        assert!(gen1 > gen0);
        assert_eq!(gen1, e2.generation);
    }
}
