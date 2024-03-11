use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::info;

use crate::audio::{AudioDataConsumer, AudioDevice, AudioManager};

pub struct AudioAnalyzer {}

impl AudioDataConsumer for AudioAnalyzer {
    fn consume_samples(&mut self, channels_samples: Vec<Vec<f32>>) {
        info!("Consuming data len {}", channels_samples[0].len());
    }
}

pub struct RtAudioEffect {
    audio_device: AudioDevice,
    analyzer: Arc<Mutex<AudioAnalyzer>>,
}

impl RtAudioEffect {
    pub fn new() -> Self {
        let audio_device = AudioDevice::new(AudioManager::get_default_loopback().unwrap()).unwrap();
        RtAudioEffect {
            audio_device,
            analyzer: Arc::new(Mutex::new(AudioAnalyzer {})),
        }
    }

    pub fn run(&mut self) {
        self.audio_device.set_callback(
            Duration::from_secs_f32(0.1),
            crate::audio::Overlap::None,
            self.analyzer.clone(),
        );
        self.audio_device.start();

        let start_time = Instant::now();
        // for tests
        while (Instant::now() - start_time) < Duration::from_secs_f32(3.0) {
            // collect new audio samples
            self.audio_device.run();
        }

        self.audio_device.stop();
    }
}

impl Default for RtAudioEffect {
    fn default() -> Self {
        Self::new()
    }
}
