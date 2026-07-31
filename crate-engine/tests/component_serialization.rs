use ecs_core::*;

#[test]
fn serialize_f32() {
    let mut data = Vec::new();

    FieldType::F32.write_value(&ComponentValue::F32(10.5), &mut data);

    assert_eq!(data.len(), 4);
}
