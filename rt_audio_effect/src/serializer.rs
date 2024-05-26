use log::error;

use crate::Serializable;

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
            }
        )*
    };
}

impl_packet_buildable_for_trivial!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

impl Serializable for String {
    fn get_bytes(&self) -> Vec<u8> {
        let mut data = (self.len() as u32).to_ne_bytes().to_vec();
        data.extend_from_slice(self.as_bytes());

        data
    }

    fn from_bytes(&mut self, bytes: &[u8]) -> u32 {
        let len = u32::from_ne_bytes(bytes[0..4].try_into().unwrap()) as usize;
        match String::from_utf8(bytes[4..4 + len].to_vec()) {
            Ok(str) => *self = str,
            Err(err) => {
                *self = String::from("");
                error!("Error while converting bytes to string: {:?}", err);
            }
        }

        len as u32 + 4
    }
}

#[derive(Debug)]
pub struct Packet {
    data: Vec<u8>,
}

impl Packet {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(mut self, data: impl Serializable) -> Self {
        self.data.extend_from_slice(&data.get_bytes());
        self
    }
}

pub struct Serializer {}

impl Serializer {
    pub fn new() -> Self {
        Serializer {}
    }

    pub fn serialize(self, packet: Packet) -> Vec<u8> {
        Vec::new()
    }

    pub fn serialize_message(&self, msg: Message) -> Packet {
        Packet::new().add(msg.index).add(msg.random)
    }
}

#[derive(Debug)]
pub struct Message {
    pub index: u32,
    pub random: f32,
}

fn dupa() {
    let msg = Message {
        index: 42,
        random: 3.14,
    };

    let serializer = Serializer::new();
    let packet = serializer.serialize_message(msg);
    println!("{:?}", packet);
}
