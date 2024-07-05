use log::error;

mod test_enum {
    use serializer::Serializable;

    #[derive(Serializable, Default, Debug, PartialEq)]
    pub enum ExampleEnum {
        #[default]
        SomeUnitFieldA,
        SomeUnnamedFieldA(String),
        SomeUnitFieldB,
        SomeUnnamedFieldB(u32),
    }

    #[test]
    fn test_field_unit_a() {
        let input_enum = ExampleEnum::SomeUnitFieldA;
        let bytes = input_enum.get_bytes();

        let output_enum = ExampleEnum::try_from_bytes(&bytes).unwrap();
        assert_eq!(output_enum, input_enum);
        assert_eq!(bytes.len(), 1); // only discriminator
        assert_eq!(bytes[0], 0); // discriminator value
    }

    #[test]
    fn test_field_unnamed_a() {
        let example_string = String::from("TEST");
        let input_enum = ExampleEnum::SomeUnnamedFieldA(example_string.clone());
        let bytes = input_enum.get_bytes();

        let output_enum = ExampleEnum::try_from_bytes(&bytes).unwrap();
        assert_eq!(output_enum, input_enum);
        assert_eq!(bytes.len(), 1 + 4 + example_string.len()); // discriminator + string length (u32) + string
        assert_eq!(bytes[0], 1); // discriminator value
        assert_eq!(
            u32::from_le_bytes(bytes[1..5].try_into().unwrap()),
            example_string.len() as u32
        ); // string length
        assert_eq!(&bytes[5..], example_string.as_bytes()); // string
    }
}
