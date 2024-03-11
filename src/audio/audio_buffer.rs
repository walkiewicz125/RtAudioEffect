use log::{debug, warn};

pub struct AudioBuffer {
    channels: u16,
    buffer_duration_in_samples: usize,
    samples: Vec<Vec<f32>>,
    new_samples_count: usize,
}

impl AudioBuffer {
    pub fn new(channels: u16, buffer_duration_in_samples: usize) -> AudioBuffer {
        AudioBuffer {
            channels,
            buffer_duration_in_samples,
            samples: vec![vec![0.0; buffer_duration_in_samples]; channels as usize],
            new_samples_count: 0,
        }
    }

    pub fn store(&mut self, data: Vec<f32>) {
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

    pub fn read_samples(
        &mut self,
        sample_count: usize,
        overlap_size: usize,
    ) -> Result<Vec<Vec<f32>>, String> {
        debug!("reading samples. sample_count: {sample_count}, overlap_size: {overlap_size}");
        assert!(
            overlap_size < sample_count,
            "sample_count have to be greater than overlap_count"
        );

        if self.new_samples_count < (sample_count - overlap_size) {
            return Err(String::from("Not enough data"));
        }

        let start_index = self.buffer_duration_in_samples - self.new_samples_count - overlap_size;
        let end_index = start_index + sample_count;

        let mut channels_samples: Vec<Vec<f32>> = vec![];

        for channel_samples in &self.samples {
            let samples = channel_samples[start_index..end_index].to_vec();
            channels_samples.push(samples);
        }

        self.new_samples_count -= sample_count - overlap_size;

        Ok(channels_samples)
    }

    fn trim_buffers(&mut self) {
        for buffer in &mut self.samples {
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

    fn distribute_into_channels(&mut self, data: Vec<f32>) -> usize {
        for i in 0..data.len() {
            let channel = i % self.channels as usize;
            self.samples[channel].push(data[i]);
        }
        data.len() / self.channels as usize
    }
}
