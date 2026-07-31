use ecs_core::*;

#[test]
fn field_column_len() {
    let mut column = FieldColumn::new("x".into(), FieldType::F32);

    column.data.extend_from_slice(&1.0f32.to_le_bytes());

    column.data.extend_from_slice(&2.0f32.to_le_bytes());

    column.data.extend_from_slice(&3.0f32.to_le_bytes());

    assert_eq!(column.len(), 3);
}
