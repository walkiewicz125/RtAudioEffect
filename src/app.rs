use std::{
    thread,
    time::{Duration, Instant},
};

use log::info;

use crate::audio::{AudioDevice, AudioManager};

pub struct RtAudioEffect {
    audio_device: AudioDevice,
}

impl RtAudioEffect {
    pub fn new() -> Self {
        let audio_device = AudioDevice::new(AudioManager::get_default_loopback().unwrap()).unwrap();
        RtAudioEffect { audio_device }
    }

    pub fn analyze(&self, channels_samples: Vec<Vec<f32>>) {}
    pub fn run(&mut self) {
        self.audio_device.set_callback(
            Duration::from_secs_f32(0.1),
            crate::audio::Overlap::None,
            |channels_samples: Vec<Vec<f32>>| self.analyze(channels_samples),
        );

        self.audio_device.start();

        let start_time = Instant::now();
        let mut readed_samples = 0;
        // for tests
        while (Instant::now() - start_time) < Duration::from_secs_f32(3.0) {
            let new_samples = self.audio_device.get_new_samples_count();
            readed_samples += new_samples;
            if new_samples > 0 {
                let _data = self.audio_device.read_samples(new_samples, 0);
            }
        }

        info!("Readed samples: {readed_samples}");

        self.audio_device.stop();
    }
}

impl Default for RtAudioEffect {
    fn default() -> Self {
        Self::new()
    }
}
