#[cfg(test)]
mod test_combined {
    use crate::Serializable;

    #[derive(Default, Serializable, Debug, PartialEq, Clone)]
    enum TestE {
        #[default]
        A,
        B,
        C(u32, String, u8),
        D(String),
    }
    #[derive(Default, Serializable, Debug, PartialEq)]
    struct TestS {
        a: String,
        b: String,
        c: u32,
        d: TestE,
    }

    #[test]
    fn test() {
        let test_data = TestS {
            a: "hello".to_string(),
            b: "world".to_string(),
            c: 42,
            d: TestE::C(11, "inner".to_string(), 22),
        };

        let bytes = test_data.to_bytes();
        let mut packet: TestS = TestS::default();
        packet.from_bytes(bytes).unwrap();

        println!("{:?}", packet);

        assert_eq!(packet, test_data);
    }
}

#[cfg(test)]
mod test_struct {
    use crate::Serializable;

    #[derive(Default, Serializable, Debug, PartialEq)]
    struct TestSInner {
        a: String,
        b: String,
        c: u32,
    }

    #[derive(Default, Serializable, Debug, PartialEq)]
    struct TestS {
        a: String,
        b: String,
        c: u32,
        d: TestSInner,
    }

    #[test]
    fn test() {
        let test_data = TestS {
            a: "hello".to_string(),
            b: "world".to_string(),
            c: 42,
            d: TestSInner {
                a: "inner".to_string(),
                b: "struct".to_string(),
                c: 24,
            },
        };

        let bytes = test_data.to_bytes();
        let mut packet: TestS = TestS::default();
        packet.from_bytes(bytes).unwrap();

        println!("{:?}", packet);

        assert_eq!(packet, test_data);
    }
}

#[cfg(test)]
mod test_enum_unnamed {
    use crate::Serializable;

    #[derive(Default, Serializable, Debug, PartialEq, Clone)]
    enum TestE {
        #[default]
        A,
        B,
        C(u32, String, u8),
        D(String),
    }

    #[test]
    fn test_a() {
        let test_data = TestE::A;
        let expected_test_data = test_data.clone();

        let bytes = test_data.to_bytes();

        // [Enum discriminant: u32]
        assert_eq!(bytes.len(), 4);

        let mut packet: TestE = TestE::default();
        packet.from_bytes(bytes).unwrap();

        assert_eq!(packet, expected_test_data);
    }

    #[test]
    fn test_b() {
        let test_data = TestE::B;
        let expected_test_data = test_data.clone();

        let bytes = test_data.to_bytes();

        // [Enum discriminant: u32]
        assert_eq!(bytes.len(), 4);

        let mut packet: TestE = TestE::default();
        packet.from_bytes(bytes).unwrap();

        assert_eq!(packet, expected_test_data);
    }

    #[test]
    fn test_c() {
        let test_data = TestE::C(
            {
                let a = 42;
                a
            },
            "hello".to_string(),
            24,
        );

        let expected_test_data = test_data.clone();

        let bytes = test_data.to_bytes();

        // [Enum discriminant: u32, field_1(A): u32, field_2(B): String(len: u32, data: [u8]), field_3(C)): u8]
        // 4 + 4 + 4 + 5(Hello) + 1 = 18
        assert_eq!(bytes.len(), 18);

        let mut packet: TestE = TestE::default();
        packet.from_bytes(bytes).unwrap();

        assert_eq!(packet, expected_test_data);
    }

    #[test]
    fn test_d() {
        let test_data = TestE::D("hello world".to_string());

        let expected_test_data = test_data.clone();

        let bytes = test_data.to_bytes();

        // [Enum discriminant: u32, String(len: u32, data: [u8])]
        // 4 + 4 + 11(Hello) = 19
        assert_eq!(bytes.len(), 19);

        let mut packet: TestE = TestE::default();
        packet.from_bytes(bytes).unwrap();

        assert_eq!(packet, expected_test_data);
    }
}

#[cfg(test)]
mod test_enum_named {
    use crate::Serializable;

    #[derive(Default, Serializable, Debug, PartialEq, Clone)]
    enum TestE {
        #[default]
        A,
        B,
        C {
            x: u32,
            y: String,
        },
        D(String),
    }

    #[test]
    fn test_a() {
        let test_data = TestE::C {
            x: { 32 },
            y: "hello".to_string(),
        };

        let expected_test_data = test_data.clone();

        let bytes = test_data.to_bytes();

        // [Enum discriminant: u32, field_C_x: u32, field_C_y: String(len: u32, data: [u8])]
        // 4 + 4 + 4 + 5(Hello) = 17
        assert_eq!(bytes.len(), 17);

        let mut packet: TestE = TestE::default();
        packet.from_bytes(bytes).unwrap();

        assert_eq!(packet, expected_test_data);
    }
}
