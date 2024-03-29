use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    audio::{AudioDevice, AudioManager, StreamParameters},
    audio_analyzer::{AnalyzerParameters, ManyChannelsSpectrums, StreamAnalyzer},
};

pub trait AudioAnalyzysProvider {
    fn get_stream_parameters(&self) -> Arc<StreamParameters>;
    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters>;
    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums;
}

pub struct AudioProcessor {
    audio_device: AudioDevice,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
}

impl AudioProcessor {
    pub fn new() -> Self {
        let mut audio_device =
            AudioDevice::new(AudioManager::get_default_loopback().unwrap()).unwrap();
        let audio_device_parameters = audio_device.get_parameters();
        let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(
            Duration::from_secs_f32(0.01),
            Duration::from_secs_f32(1.0),
            4800,
            audio_device_parameters,
        )));

        audio_device.add_stream_consumer(analyzer.clone(), Some(String::from("Spectrum Analyzer")));

        AudioProcessor {
            audio_device,
            analyzer,
        }
    }

    pub fn start(&mut self) {
        self.audio_device.start();
    }

    pub fn update(&mut self) {
        self.audio_device.update();
    }

    pub fn stop(&mut self) {
        self.audio_device.stop();
    }
}

impl AudioAnalyzysProvider for AudioProcessor {
    fn get_stream_parameters(&self) -> Arc<StreamParameters> {
        self.audio_device.get_parameters().clone()
    }

    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters> {
        self.analyzer.lock().unwrap().get_analyzer_parameters()
    }

    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.analyzer.lock().unwrap().get_latest_spectrum()
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new()
    }
}
