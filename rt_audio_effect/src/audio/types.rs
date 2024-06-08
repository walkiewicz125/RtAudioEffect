use std::ops::{Index, IndexMut};

pub type Sample = f32;

#[derive(Clone)]
pub struct ChannelSamples(Vec<Sample>);

pub struct MixedChannelsSamples(Vec<Sample>);

impl ChannelSamples {
    pub fn inner_mut(&mut self) -> &mut Vec<Sample> {
        &mut self.0
    }

    pub fn inner(&self) -> &Vec<Sample> {
        &self.0
    }
}

impl From<Vec<Sample>> for ChannelSamples {
    fn from(samples: Vec<Sample>) -> Self {
        ChannelSamples(samples)
    }
}
impl Index<usize> for ChannelSamples {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for ChannelSamples {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl MixedChannelsSamples {
    pub fn inner(&self) -> &Vec<Sample> {
        &self.0
    }
}

impl From<Vec<Sample>> for MixedChannelsSamples {
    fn from(samples: Vec<Sample>) -> Self {
        MixedChannelsSamples(samples)
    }
}
