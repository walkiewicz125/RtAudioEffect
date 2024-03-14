use std::{sync::Arc, time::Duration};

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::audio::{AudioBuffer, AudioDeviceParameters, AudioStreamConsumer};

use super::SpectrumAnalyzer;

pub struct StreamAnalyzer {
    spectrum_analyzer: SpectrumAnalyzer,
    spectrum_width: usize,
    buffer: AudioBuffer,
    refresh_time: Duration,
    refresh_time_in_samples: usize,
    history_duration: Duration,
}

impl AudioStreamConsumer for StreamAnalyzer {
    fn process_new_samples(&mut self, audio_samples: Vec<f32>) {
        self.buffer.store(audio_samples);

        if self.buffer.get_new_samples_count() >= self.refresh_time_in_samples {
            while let Ok(new_samples) = self
                .buffer
                .read_new_samples(self.refresh_time_in_samples, self.spectrum_width)
            {
                if new_samples.len() > 0 {
                    self.spectrum_analyzer.analyze(&new_samples[0]);
                    println!("lalalalalal");
                }
            }
        }
    }
}

impl StreamAnalyzer {
    pub fn new(
        refresh_time: Duration,
        history_duration: Duration,
        spectrum_width: usize,
        audio_device_parameters: AudioDeviceParameters,
    ) -> StreamAnalyzer {
        let refresh_time_in_samples =
            (audio_device_parameters.sample_rate as f32 * refresh_time.as_secs_f32()) as usize;

        StreamAnalyzer {
            spectrum_analyzer: SpectrumAnalyzer::new(spectrum_width),
            spectrum_width,
            buffer: AudioBuffer::new(
                audio_device_parameters.channels,
                audio_device_parameters.sample_rate as usize,
            ),
            refresh_time_in_samples,
            refresh_time,
            history_duration,
        }
    }
}
