use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use log::info;

use crate::{
    audio::{AudioDevice, AudioManager, AudioStreamConsumer},
    audio_analyzer::StreamAnalyzer,
};

pub struct RtAudioEffect {
    audio_device: AudioDevice,
    analyzer: Arc<Mutex<dyn AudioStreamConsumer>>,
}

impl RtAudioEffect {
    pub fn new() -> Self {
        let audio_device = AudioDevice::new(AudioManager::get_default_loopback().unwrap()).unwrap();
        let audio_device_parameters = audio_device.get_parameters();
        RtAudioEffect {
            audio_device,
            analyzer: Arc::new(Mutex::new(StreamAnalyzer::new(
                Duration::from_secs_f32(0.01),
                Duration::from_secs_f32(1.0),
                1024,
                audio_device_parameters,
            ))),
        }
    }

    pub fn run(&mut self) {
        self.audio_device.add_stream_consumer(
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
