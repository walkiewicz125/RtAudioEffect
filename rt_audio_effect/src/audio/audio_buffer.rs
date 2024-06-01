use std::{
    ops::{Index, IndexMut},
    sync::Arc,
    time::Duration,
};

use log::{trace, warn};

use crate::audio_analyzer::MultiChannel;

use super::StreamParameters;

pub type Sample = f32;

pub struct MixedChannelsSamples(Vec<Sample>);
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
#[derive(Clone)]
pub struct ChannelSamples(Vec<Sample>);
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
pub struct AudioBuffer {
    channels: u16,
    buffer_duration_in_samples: usize,
    channels_buffers: MultiChannel<ChannelSamples>,
    new_samples_count: usize,
}

impl AudioBuffer {
    pub fn new(stream_parameters: Arc<StreamParameters>, buffer_duration: Duration) -> AudioBuffer {
        let buffer_duration_in_samples =
            (stream_parameters.sample_rate as f32 * buffer_duration.as_secs_f32()) as usize;

        let empty_channels_buffers = MultiChannel::new(
            stream_parameters.channels as usize,
            ChannelSamples::from(vec![0.0; buffer_duration_in_samples]),
        );
        AudioBuffer {
            channels: stream_parameters.channels,
            buffer_duration_in_samples,
            channels_buffers: empty_channels_buffers.into(),
            new_samples_count: 0,
        }
    }

    pub fn store(&mut self, data: MixedChannelsSamples) {
        let new_samples = self.distribute_into_channels(data);
        self.trim_buffers();

        self.new_samples_count += new_samples;
        if self.new_samples_count > self.buffer_duration_in_samples {
            let overrun = self.new_samples_count - self.buffer_duration_in_samples;
            self.new_samples_count = self.buffer_duration_in_samples;

            warn!("Buffer overrun by: {overrun:#?}");
        }
    }

    pub fn get_new_samples_count(&self) -> usize {
        self.new_samples_count
    }

    pub fn read_new_samples(
        &mut self,
        new_samples: usize,
        total_sample_count: usize,
    ) -> Result<MultiChannel<ChannelSamples>, String> {
        trace!("Getting {total_sample_count} for all channels, with new samples: {new_samples}");

        assert!(
            new_samples <= total_sample_count,
            "Total_sample_count have to be greater than new_samples"
        );

        assert!(
            total_sample_count <= self.buffer_duration_in_samples,
            "Total_sample_count have to be lesser than buffer_duration_in_samples"
        );

        if self.new_samples_count < new_samples {
            return Err(String::from("Not enough new data"));
        }

        let start_index =
            self.buffer_duration_in_samples - self.new_samples_count - total_sample_count
                + new_samples;
        let end_index = start_index + total_sample_count;

        let mut channels_samples: Vec<ChannelSamples> = Vec::new();

        for channel_samples in self.channels_buffers.inner_mut() {
            let samples = channel_samples.inner_mut()[start_index..end_index].to_vec();
            channels_samples.push(samples.into());
        }

        self.new_samples_count -= new_samples;

        Ok(channels_samples.into())
    }

    fn trim_buffers(&mut self) {
        for (channel, buffer) in self.channels_buffers.inner_mut().iter_mut().enumerate() {
            if buffer.inner_mut().len() > self.buffer_duration_in_samples {
                let oversize = buffer.inner_mut().len() - self.buffer_duration_in_samples;
                buffer.inner_mut().drain(0..oversize);
                trace!("Trimming buffer[{channel}] by {oversize}");
            }
            assert!(
                buffer.inner_mut().len() <= self.buffer_duration_in_samples,
                "buffer {channel} didn't shrink"
            );
        }
    }

    fn distribute_into_channels(&mut self, data: MixedChannelsSamples) -> usize {
        let new_samples_per_channel = data.inner().len() / self.channels as usize;
        trace!(
            "Distributing samples into separate channels. Channel count {}, new sample count per channel {}",
            self.channels,
            new_samples_per_channel
        );

        for (sample_number, sample) in data.inner().into_iter().enumerate() {
            let channel = sample_number % self.channels as usize;
            self.channels_buffers
                .get_channel_mut(channel)
                .inner_mut()
                .push(*sample);
        }

        new_samples_per_channel
    }
}
