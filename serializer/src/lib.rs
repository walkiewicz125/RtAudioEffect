use log::error;
extern crate serializer_macro;
pub use serializer_macro::Serializable;

pub trait Serializable {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, String>
    where
        Self: Sized;
    fn get_bytes(&self) -> Vec<u8>;
    fn from_bytes(&mut self, bytes: &[u8]) -> u32;
}

macro_rules! impl_packet_buildable_for_trivial {
    ($($t:ty),*) => {
        $(
            impl Serializable for $t {
                fn get_bytes(&self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }

                fn from_bytes(&mut self, bytes: &[u8]) -> u32 {
                    let size = std::mem::size_of::<Self>();
                    *self = Self::from_ne_bytes(bytes[0..size].try_into().unwrap());
                    size as u32
                }

                fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
                    let mut instance = Self::default();
                    instance.from_bytes(bytes);

                    Ok(instance)
                }
            }
        )*
    };
}

impl_packet_buildable_for_trivial!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

impl Serializable for String {
    // [len: u32:NE; data: [u8]]

    fn get_bytes(&self) -> Vec<u8> {
        let mut data = (self.len() as u32).to_ne_bytes().to_vec();
        data.extend_from_slice(self.as_bytes());

        data
    }

    fn from_bytes(&mut self, bytes: &[u8]) -> u32 {
        let len = u32::from_ne_bytes(bytes[0..4].try_into().unwrap()) as usize;
        let bytes = &bytes[4..4 + len];
        match String::from_utf8(bytes.to_vec()) {
            Ok(str) => *self = str,
            Err(err) => {
                *self = String::from("");
                error!("Error while converting bytes to string: {:?}", err);
            }
        }

        bytes.len() as u32
    }
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let mut instance = String::from("");
        instance.from_bytes(bytes);

        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use crate::Serializable;
    #[test]
    fn test_string() {
        let s = String::from("Hello, World!");

        let bytes = s.get_bytes();
        let mut s2 = String::from("");
        s2.from_bytes(&bytes);

        assert_eq!(s, s2);
    }

    // generate tests for types: i8, u8, i16, u16, i32, u32, i64, u64, f32, f64
    macro_rules! test_trivial_ints {
        ($($t:ident),*) => {
            $(
                #[test]
                fn $t() {
                    let v: $t = 42;

                    let bytes = v.get_bytes();
                    let mut v2: $t = 0;
                    v2.from_bytes(&bytes);

                    assert_eq!(v, v2);
                    assert_eq!(bytes.len(), std::mem::size_of::<$t>());
                }
            )*
        };
    }
    macro_rules! test_trivial_floats {
        ($($t:ident),*) => {
            $(
                #[test]
                fn $t() {
                    let v: $t = 42.0;

                    let bytes = v.get_bytes();
                    let mut v2: $t = 0.0;
                    v2.from_bytes(&bytes);

                    assert_eq!(v, v2);
                    assert_eq!(bytes.len(), std::mem::size_of::<$t>());
                }
            )*
        };
    }

    test_trivial_ints!(i8, u8, i16, u16, i32, u32, i64, u64);
    test_trivial_floats!(f32, f64);
}
