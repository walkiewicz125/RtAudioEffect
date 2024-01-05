use super::AudioHost;
use cpal;
use std::time::Duration;

pub struct AudioAnalyzer {
    pub host: AudioHost,
    pub spectrum_width: usize,
}

impl AudioAnalyzer {
    pub fn new_default_loopback(spectrum_width: usize) -> Result<AudioAnalyzer, String> {
        let available_hosts = cpal::available_hosts();

        if available_hosts.is_empty() {
            return Err(String::from("No host devices found"));
        }

        match AudioHost::new(available_hosts[0], spectrum_width) {
            Ok(host) => Ok(AudioAnalyzer {
                host,
                spectrum_width,
            }),
            Err(err) => return Err(String::from(err)),
        }
    }

    pub fn start(&self) {
        self.host.start();
    }

    pub fn stop(&self) {
        self.host.stop();
    }

    pub fn get_last_left_channel(&mut self) -> Vec<f32> {
        let duration =
            Duration::from_secs_f32(self.spectrum_width as f32 / self.host.sample_rate as f32);
        self.host.peek_channel(0, duration)
    }

    pub fn get_last_left_channel_mean_spectrum(&mut self) -> Vec<f32> {
        self.host.peek_mean_spectrum()
    }
    pub fn get_last_left_channel_spectrum(&mut self) -> Vec<f32> {
        self.host.peek_spectrum()
    }

    pub fn set_averaging_constant(&mut self, averaging_constant: f32) {
        self.host.set_averaging_constant(averaging_constant);
    }

    pub fn get_averaging_constant(&self) -> f32 {
        self.host.get_averaging_constant()
    }

    pub fn set_fft_length(&mut self, fft_length: u32) {
        self.spectrum_width = fft_length as usize;
        self.host.set_spectrum_width(fft_length as usize);
    }

    pub fn get_fft_length(&self) -> usize {
        self.host.spectrum_width
    }
}
