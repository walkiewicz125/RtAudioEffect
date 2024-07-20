extern crate serializer_macro;

pub use serializer_macro::Serializable;

pub trait Serializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String>
    where
        Self: Sized;
}

impl Serializable for String {
    // The first 4 bytes are the length of the string
    // The rest of the bytes are the string itself

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = (self.len() as u32).to_ne_bytes().to_vec();
        bytes.extend_from_slice(self.as_bytes());
        bytes
    }

    fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String> {
        if bytes.len() < 4 {
            return Err(format!(
                "Invalid length of bytes. Expected at least 4, got: {}",
                bytes.len(),
            ));
        }
        let size = u32::from_ne_bytes(bytes[0..4].try_into().map_err(|e| format!("Error parsing bytes: {}", e))?);
        *self = String::from_utf8(bytes[4..4+size as usize].to_vec())
            .map_err(|e| format!("Error parsing string: {}", e))?;
        Ok(4 + self.len() as u32)
    }
}

macro_rules! impl_serializable_for_trivial {
    ($($t:ty),*) => {
        $(
            impl Serializable for $t {
                fn to_bytes(&self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }

                fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String> {
                    if bytes.len() != std::mem::size_of::<Self>() {

                    }
                    let size = std::mem::size_of::<Self>();
                    *self = <$t>::from_ne_bytes(bytes[0..size].try_into().map_err(|e| format!("Error parsing bytes: {}", e))?);
                    Ok(size as u32)
                }
            }
        )*
    };
}

impl_serializable_for_trivial!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

#[cfg(test)]
mod tests;
