extern crate serializer_macro;
pub use serializer_macro::ByteMessage;

use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub trait ByteMessage {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(&mut self, bytes: Vec<u8>) -> Result<u32, String>
    where
        Self: Sized;
}

pub struct ByteMessagePort<T> {
    stream: TcpStream,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ByteMessage + Default> ByteMessagePort<T> {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, message: impl ByteMessage) -> Result<(), String> {
        let bytes: Vec<u8> = message.to_bytes();

        let len_buffer = (bytes.len() as u32).to_ne_bytes();
        self.stream
            .write_all(&len_buffer)
            .map_err(|err| format!("Failed to send message length: {:?}", err))?;
        self.stream
            .write_all(&bytes)
            .map_err(|err| format!("Failed to send message: {:?}", err))
    }

    pub fn recv(&mut self) -> Result<T, String> {
        let mut len_buffer: [u8; 4] = [0; 4];
        self.stream
            .read_exact(&mut len_buffer)
            .map_err(|err| format!("Failed to read message length: {:?}", err))?;
        let len = u32::from_ne_bytes(len_buffer) as usize;

        let mut data_buffer = vec![0; len];
        self.stream
            .read_exact(&mut data_buffer)
            .map_err(|err| format!("Failed to read message data: {:?}", err))?;

        let mut message = T::default();
        message.from_bytes(data_buffer)?;
        Ok(message)
    }
}

impl ByteMessage for String {
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
        let size = u32::from_ne_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|e| format!("Error parsing bytes: {}", e))?,
        );
        *self = String::from_utf8(bytes[4..4 + size as usize].to_vec())
            .map_err(|e| format!("Error parsing string: {}", e))?;
        Ok(4 + self.len() as u32)
    }
}

macro_rules! impl_byte_message_for_trivial {
    ($($t:ty),*) => {
        $(
            impl ByteMessage for $t {
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

impl_byte_message_for_trivial!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

#[cfg(test)]
mod tests;
