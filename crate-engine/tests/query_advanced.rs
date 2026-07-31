use ecs_core::*;

fn setup() -> (World, u32, u32, u32) {
    let mut world = World::new();
    let pos = world.register_component(
        ComponentBuilder::new("Position").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let vel = world.register_component(
        ComponentBuilder::new("Velocity").field("x", FieldType::F32).field("y", FieldType::F32).build()
    );
    let tag = world.register_component(
        ComponentBuilder::new("Tag").field("value", FieldType::U32).build()
    );
    (world, pos, vel, tag)
}

#[test]
fn query_multi_component() {
    let (mut world, pos, vel, _) = setup();

    // Entity with pos+vel
    world.spawn(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)]),
        ComponentInstance::new(vel, vec![ComponentValue::F32(0.1), ComponentValue::F32(0.2)]),
    ]);
    // Entity with pos only
    world.spawn(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)]),
    ]);

    let result = world.query(&[pos, vel]);
    assert_eq!(result.total_rows(), 1, "Only 1 entity has both pos+vel");
}

#[test]
fn query_exclude() {
    let (mut world, pos, vel, tag) = setup();

    world.spawn(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)]),
        ComponentInstance::new(vel, vec![ComponentValue::F32(0.1), ComponentValue::F32(0.2)]),
    ]);
    world.spawn(vec![
        ComponentInstance::new(pos, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)]),
        ComponentInstance::new(vel, vec![ComponentValue::F32(0.3), ComponentValue::F32(0.4)]),
        ComponentInstance::new(tag, vec![ComponentValue::U32(1)]),
    ]);

    // Get pos+vel but NOT tag
    let result = world.query_with_exclude(&[pos, vel], &[tag]);
    assert_eq!(result.total_rows(), 1);
}

#[test]
fn query_total_rows_correct() {
    let (mut world, pos, _, _) = setup();

    for i in 0..10 {
        world.spawn(vec![ComponentInstance::new(pos, vec![
            ComponentValue::F32(i as f32), ComponentValue::F32(0.0)
        ])]);
    }

    let result = world.query(&[pos]);
    assert_eq!(result.total_rows(), 10);
}

#[test]
fn query_column_bytes_len() {
    let (mut world, pos, _, _) = setup();

    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(1.0), ComponentValue::F32(2.0)])]);
    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(3.0), ComponentValue::F32(4.0)])]);

    let cols = world.query_column(pos);
    assert_eq!(cols.len(), 1); // 1 chunk
    assert_eq!(cols[0].len(), 16); // 2 entities × 8 bytes
}

#[test]
fn query_after_despawn_reduces_rows() {
    let (mut world, pos, _, _) = setup();

    let entities: Vec<Entity> = (0..5).map(|i| {
        world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(i as f32), ComponentValue::F32(0.0)])])
    }).collect();

    let result = world.query(&[pos]);
    assert_eq!(result.total_rows(), 5);

    world.despawn(entities[2]).unwrap();

    let result = world.query(&[pos]);
    assert_eq!(result.total_rows(), 4);
}

#[test]
fn query_iter_rows() {
    let (mut world, pos, _, _) = setup();

    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(10.0), ComponentValue::F32(20.0)])]);
    world.spawn(vec![ComponentInstance::new(pos, vec![ComponentValue::F32(30.0), ComponentValue::F32(40.0)])]);

    let result = world.query(&[pos]);
    let mut xs = Vec::new();
    for (chunk, row) in result.iter_rows() {
        let bytes = chunk.columns[0].row(row);
        xs.push(f32::from_le_bytes(bytes[0..4].try_into().unwrap()));
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(xs, vec![10.0, 30.0]);
}
