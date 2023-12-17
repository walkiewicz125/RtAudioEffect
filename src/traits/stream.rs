pub struct NormalizedMixedChanelData {
    pub stream_data: Vec<f32>,
    pub num_of_chanels: i8,
    pub sampling_frequency: f32,
}

pub enum StreamData {
    NormalizedMixedChanel(NormalizedMixedChanelData),
}
pub trait Stream {
    fn fetch_data(&mut self) -> StreamData;
}
