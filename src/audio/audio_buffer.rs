use std::sync::Arc;

use log::{debug, warn};

use super::StreamParameters;

pub type Sample = f32;
pub type MixedChannelsSamples = Vec<Sample>;
pub type OneChannelSamples = Vec<Sample>;
pub type ManyChannelsSamples = Vec<OneChannelSamples>;

pub struct AudioBuffer {
    channels: u16,
    buffer_duration_in_samples: usize,
    channels_buffers: ManyChannelsSamples,
    new_samples_count: usize,
}

impl AudioBuffer {
    pub fn new(
        stream_parameters: Arc<StreamParameters>,
        buffer_duration_in_samples: usize,
    ) -> AudioBuffer {
        let empty_channels_buffers =
            vec![vec![0.0; buffer_duration_in_samples]; stream_parameters.channels as usize];
        AudioBuffer {
            channels: stream_parameters.channels,
            buffer_duration_in_samples,
            channels_buffers: empty_channels_buffers,
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
    ) -> Result<ManyChannelsSamples, String> {
        debug!(
            "reading samples. new_samples: {new_samples}, total_sample_count: {total_sample_count}"
        );

        assert!(
            new_samples < total_sample_count,
            "total_sample_count have to be greater than new_samples"
        );

        assert!(
            total_sample_count < self.buffer_duration_in_samples,
            "total_sample_count have to be lesser than buffer_duration_in_samples"
        );

        if self.new_samples_count < new_samples {
            return Err(String::from("Not enough new data"));
        }

        let start_index =
            self.buffer_duration_in_samples - self.new_samples_count - total_sample_count
                + new_samples;
        let end_index = start_index + total_sample_count;

        let mut channels_samples: ManyChannelsSamples = ManyChannelsSamples::default();

        for channel_samples in &self.channels_buffers {
            let samples = channel_samples[start_index..end_index].to_vec();
            channels_samples.push(samples);
        }

        self.new_samples_count -= new_samples;

        Ok(channels_samples)
    }

    fn trim_buffers(&mut self) {
        for buffer in &mut self.channels_buffers {
            if buffer.len() > self.buffer_duration_in_samples {
                let oversize = buffer.len() - self.buffer_duration_in_samples;
                buffer.drain(0..oversize);
            }
            assert!(
                buffer.len() <= self.buffer_duration_in_samples,
                "buffer didn't shrink"
            );
        }
    }

    fn distribute_into_channels(&mut self, data: MixedChannelsSamples) -> usize {
        for i in 0..data.len() {
            let channel = i % self.channels as usize;
            self.channels_buffers[channel].push(data[i]);
        }
        data.len() / self.channels as usize
    }
}
