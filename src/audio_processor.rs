use std::{
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use crate::{
    audio::{
        audio_stream_source::AudioStreamSource, AudioManager, AudioStreamReceiver,
        MixedChannelsSamples, StreamParameters,
    },
    audio_analyzer::{AnalyzerParameters, ManyChannelsSpectrums, StreamAnalyzer},
};

pub trait AudioAnalyzysProvider {
    fn get_stream_parameters(&self) -> Arc<StreamParameters>;
    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters>;
    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums;
}

pub struct AudioStream {
    stream_source: AudioStreamSource,
    pub stream_receiver: Arc<Mutex<AudioStreamReceiver>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
}

impl AudioStream {
    pub fn new() -> Self {
        let (channel_tx, channel_rx) = mpsc::channel::<MixedChannelsSamples>();

        let mut stream_source =
            AudioStreamSource::new(AudioManager::get_default_loopback().unwrap(), channel_tx)
                .unwrap();
        let parameters = stream_source.get_parameters();
        let mut stream_receiver = AudioStreamReceiver::new(parameters.clone(), channel_rx).unwrap();

        let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(
            Duration::from_secs_f32(0.01),
            Duration::from_secs_f32(1.0),
            4800,
            parameters,
        )));

        stream_receiver
            .add_stream_consumer(analyzer.clone(), Some(String::from("Spectrum Analyzer")));

        AudioStream {
            stream_source,
            stream_receiver: Arc::new(Mutex::new(stream_receiver)),
            analyzer,
        }
    }

    pub fn start(&mut self) {
        self.stream_source.start();
    }

    pub fn stop(&mut self) {
        self.stream_source.stop();
    }
}

impl AudioAnalyzysProvider for AudioStream {
    fn get_stream_parameters(&self) -> Arc<StreamParameters> {
        self.stream_source.get_parameters().clone()
    }

    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters> {
        self.analyzer.lock().unwrap().get_analyzer_parameters()
    }

    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.analyzer.lock().unwrap().get_latest_spectrum()
    }
}

impl Default for AudioStream {
    fn default() -> Self {
        Self::new()
    }
}
