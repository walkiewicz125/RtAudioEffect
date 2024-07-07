// fn a() -> Result<String, String> {
//     let bytes: Vec<u8> = Vec::new();

//     let a: String = WireData::from_bytes(bytes)?;

//     Ok(a)
// }

mod test_struct {
    use std::clone;

    use serializer::{DataView, WireData, WireDataSerializer};
    use serializer_macro::WireDataSerializer;

    #[derive(WireDataSerializer, Default, Debug, PartialEq, Clone)]
    pub struct ExampleStruct {
        field_a: String,
        field_b: u32,
    }

    #[test]
    fn test_struct() {
        let instance = ExampleStruct {
            field_a: "Hello, World!".to_string(),
            field_b: 42,
        };

        let bytes = WireData::get_bytes(instance.clone());
        let deserialized_instance = WireData::from_bytes(bytes).unwrap();

        assert_eq!(instance, deserialized_instance);
    }
}
