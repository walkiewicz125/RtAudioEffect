use std::time::Duration;

pub struct AudioBuffer {
    sample_rate: u32,
    channels: u16,
    buffer_duration: Duration,
    buffer_duration_in_samples: usize,

    buffers: Vec<Vec<f32>>,
    new_sample_count: u32,
}

// public interface
impl AudioBuffer {
    pub fn new(sample_rate: u32, channels: u16, buffer_duration: Duration) -> AudioBuffer {
        let buffer_duration_in_samples: usize =
            (sample_rate as f32 * buffer_duration.as_secs_f32()) as usize;
        AudioBuffer {
            sample_rate,
            channels,
            buffer_duration,
            buffer_duration_in_samples,
            buffers: vec![vec![0.0; buffer_duration_in_samples]; channels as usize],
            new_sample_count: 0,
        }
    }

    pub fn store(&mut self, data: Vec<f32>) {
        let new_samples = self.split_channels(data);
        self.trim_buffers();
        self.new_sample_count += new_samples;
        if self.new_sample_count > self.buffer_duration_in_samples as u32 {
            self.new_sample_count = self.buffer_duration_in_samples as u32;
        }
    }

    pub fn peek_channel(&self, channel: u16) -> Vec<f32> {
        self.buffers[channel as usize].clone()
    }

    pub fn fetch_with_overlap(
        &mut self,
        channel: u16,
        sample_count: usize,
        overlap_count: usize,
    ) -> Vec<f32> {
        assert!(
            overlap_count < sample_count,
            "sample_count have to be greater than overlap_count"
        );

        if self.new_sample_count < (sample_count - overlap_count) as u32 {
            return vec![];
        }

        if let Some(v) = self
            .buffer_duration_in_samples
            .checked_sub(self.new_sample_count as usize)
        {
            if let Some(v2) = v.checked_sub(overlap_count) {
                let end_index = v2 + sample_count;
                let data = self.buffers[channel as usize][v2..end_index].to_vec();
                self.new_sample_count -= (sample_count - overlap_count) as u32;
                return data;
            } else {
                let end_index = v + sample_count;
                let data = self.buffers[channel as usize][v..end_index].to_vec();
                self.new_sample_count -= (sample_count - overlap_count) as u32;
                return data;
            }
        }

        return vec![];
    }
}

// private implementation
impl AudioBuffer {
    fn trim_buffers(&mut self) {
        for buffer in &mut self.buffers {
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

    fn split_channels(&mut self, data: Vec<f32>) -> u32 {
        for i in 0..data.len() {
            let channel = i % self.channels as usize;
            self.buffers[channel].push(data[i]);
        }
        data.len() as u32 / self.channels as u32
    }
}
