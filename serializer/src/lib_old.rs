pub mod WireData {
    use crate::{DataView, Serializable};

    pub fn get_bytes<T: Serializable>(value: T) -> Vec<u8> {
        value.get_bytes()
    }

    pub fn from_bytes<T: Serializable>(bytes: Vec<u8>) -> Result<T, String> {
        let mut view = DataView::new(bytes);
        let ret = T::from_bytes(&mut view);

        match ret {
            Ok(instance) => {
                if view.are_all_bytes_consumed() {
                    return Err(format!(
                        "Not all bytes were consumed while deserializing. Consumed: {}, Total: {}",
                        view.position,
                        view.bytes.len()
                    ));
                } else {
                    return Ok(instance);
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
extern crate serializer_macro;

trait Serializable {
    fn get_bytes(&self) -> Vec<u8>;
    fn from_bytes(data_view: &mut DataView) -> Result<Self, String>
    where
        Self: Sized;
}

struct DataView {
    bytes: Vec<u8>,
    position: usize,
}

impl DataView {
    fn new(bytes: Vec<u8>) -> DataView {
        DataView { bytes, position: 0 }
    }

    fn read(&mut self, size: usize) -> Result<&[u8], String> {
        if self.position + size > self.bytes.len() {
            return Err(format!(
                "Not enough bytes to read. Position: {}, Size: {}, Total: {}",
                self.position,
                size,
                self.bytes.len()
            ));
        }

        let data = &self.bytes[self.position..self.position + size];
        self.position += size;
        Ok(data)
    }

    fn read_sized<T: Sized>(&mut self) -> Result<&[u8], String> {
        let size = std::mem::size_of::<T>();

        self.read(size)
    }

    fn are_all_bytes_consumed(&self) -> bool {
        self.position == self.bytes.len()
    }
}

macro_rules! impl_packet_buildable_for_trivial {
    ($($t:ty),*) => {
        $(
            impl Serializable for $t {
                fn get_bytes(&self) -> Vec<u8> {
                    self.to_ne_bytes().to_vec()
                }

                fn from_bytes(data_view: &mut DataView) -> Result<Self, String> {
                    let data = data_view.read_sized::<Self>()?;
                    let instance = Self::from_ne_bytes(data.try_into().unwrap());
                    Ok(instance)
                }
            }
        )*
    };
}

impl_packet_buildable_for_trivial!(i8, u8, i16, u16, i32, u32, i64, u64, f32, f64);

impl Serializable for String {
    // [len: LE u32; str: [u8]]
    fn get_bytes(&self) -> Vec<u8> {
        let mut data = (self.len() as u32).to_ne_bytes().to_vec();
        data.extend_from_slice(self.as_bytes());

        data
    }

    fn from_bytes(data_view: &mut DataView) -> Result<Self, String> {
        let len = u32::from_bytes(data_view)?;
        let data = data_view.read(len as usize)?;

        String::from_utf8(data.to_vec())
            .map_err(|err| format!("Error while converting bytes to string: {:?}", err))
    }
}
