use ecs_core::*;

#[test]
fn build_from_schema() {
    let schema = ComponentBuilder::new("Position")
        .field("x", FieldType::F32)
        .field("y", FieldType::F32)
        .build();

    let column = ComponentColumn::from_schema(&schema);

    assert_eq!(column.fields.len(), 2);

    assert_eq!(column.fields[0].field_name, "x");

    assert_eq!(column.fields[1].field_name, "y");
}
