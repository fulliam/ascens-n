use ecs_core::*;

fn entity(id: u32) -> Entity {
    Entity { id, generation: 0 }
}

#[test]
fn query_near_returns_only_entities_within_radius() {
    let mut grid = SpatialGrid::new(100.0, 1000.0, 1000.0);

    let center = entity(1); // distance 0 from query point
    let just_inside = entity(2); // distance 49, radius is 50
    let just_outside = entity(3); // distance 51, radius is 50
    let far_away = entity(4); // nowhere near the query point or its cell neighborhood

    grid.rebuild(vec![
        (center, 500.0, 500.0),
        (just_inside, 549.0, 500.0),
        (just_outside, 551.0, 500.0),
        (far_away, 10.0, 10.0),
    ]);

    let mut found = grid.query_near(500.0, 500.0, 50.0);
    found.sort_by_key(|e| e.id);

    assert_eq!(found, vec![center, just_inside]);
    assert_eq!(grid.query_near_count(500.0, 500.0, 50.0), 2);
}

#[test]
fn query_near_on_empty_grid_returns_empty_and_does_not_panic() {
    let grid = SpatialGrid::new(100.0, 1000.0, 1000.0);

    assert!(grid.query_near(500.0, 500.0, 50.0).is_empty());
    assert_eq!(grid.query_near_count(500.0, 500.0, 50.0), 0);

    // Also exercise a query whose cell range falls entirely outside the
    // grid's bounds (e.g. a huge radius from the corner) — must not panic.
    assert!(grid.query_near(-10_000.0, -10_000.0, 5.0).is_empty());
}

#[test]
fn rebuilding_with_fewer_entities_does_not_leave_stale_matches() {
    let mut grid = SpatialGrid::new(100.0, 1000.0, 1000.0);

    // First rebuild: five entities clustered at the same point.
    let first_gen: Vec<Entity> = (0..5).map(entity).collect();
    grid.rebuild(first_gen.iter().map(|&e| (e, 500.0, 500.0)));
    assert_eq!(grid.query_near_count(500.0, 500.0, 10.0), 5);

    // Second rebuild: only two entities survive, at the same spot. This is
    // exactly the case that would catch a "bucket never actually cleared"
    // bug in the reuse-not-reallocate rebuild path — if `.clear()` were
    // missing (or replaced with a no-op), the first generation's five
    // entities would still show up here.
    let second_gen: Vec<Entity> = (100..102).map(entity).collect();
    grid.rebuild(second_gen.iter().map(|&e| (e, 500.0, 500.0)));

    let mut found = grid.query_near(500.0, 500.0, 10.0);
    found.sort_by_key(|e| e.id);
    assert_eq!(found, second_gen);
    assert_eq!(grid.query_near_count(500.0, 500.0, 10.0), 2);

    // None of the first generation's ids should still be reachable.
    for e in &first_gen {
        assert!(!found.contains(e));
    }
}

#[test]
fn out_of_bounds_positions_are_clamped_not_dropped_or_panicking() {
    let mut grid = SpatialGrid::new(100.0, 1000.0, 1000.0);

    // Well outside the nominal [0, 1000] world on both axes.
    let stray = entity(7);
    grid.rebuild(vec![(stray, -500.0, 5_000.0)]);

    // Bucket placement clamps to the grid's edge cell (no panic, no silent
    // drop) — a query whose own cell range clamps to that same edge cell,
    // centered on the entity's real (unclamped) position, still finds it
    // via the precise-distance filter running on the real coordinates.
    let found = grid.query_near(-500.0, 5_000.0, 50.0);
    assert!(found.contains(&stray));

    // A query far from that edge cell, deep inside the nominal world,
    // must not spuriously match it.
    let unrelated = grid.query_near(500.0, 500.0, 50.0);
    assert!(!unrelated.contains(&stray));
}

#[test]
fn cell_size_constant_is_reasonable_and_grid_dimensions_scale_with_world_size() {
    assert_eq!(CELL_SIZE, 160.0);

    let grid = SpatialGrid::new(CELL_SIZE, 4600.0, 4600.0);
    // ceil(4600/160) = 29 cells per axis, matching the JS game's own grid
    // at the same world size and cell size.
    assert_eq!(grid.cell_count(), 29 * 29);
}
