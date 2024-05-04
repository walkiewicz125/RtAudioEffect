use std::fmt::{Debug, Display};

#[derive(Clone)]
pub struct StreamParameters {
    pub sample_rate: u32,
    pub channels: u16,
}

impl Display for StreamParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StreamParameters [sample_rate: {}, channels: {}]",
            self.sample_rate, self.channels
        )
    }
}
