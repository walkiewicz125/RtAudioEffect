use super::{audio_streamer::AudioStreamer, stream_analyzer::StreamAnalyzer, AudioBuffer};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId, Stream,
};
use egui::mutex::Mutex;
use std::{sync::Arc, time::Duration};

pub struct AudioHost {
    pub host_id: HostId,
    pub sample_rate: u32,
    pub channels: u16,
    pub stream: Stream,
    pub buffer: Arc<Mutex<AudioBuffer>>,
    pub analyzer: Arc<Mutex<StreamAnalyzer>>,

    pub spectrum_width: usize,
    pub buffer_duration: Duration,
    pub buffer_duration_in_samples: usize,
}

impl AudioHost {
    pub fn new(host_id: HostId, spectrum_width: usize) -> Result<AudioHost, &'static str> {
        let Ok(host) = cpal::host_from_id(host_id) else {
            return Err("Failed to find Host");
        };

        let Some(device) = host.default_output_device() else {
            return Err("Failed to find output device");
        };

        let Ok(config) = device.default_output_config() else {
            return Err("Failed to get default config");
        };

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        println!("Sample rate: {}, channels: {}", sample_rate, channels);

        let cpal::SampleFormat::F32 = config.sample_format() else {
            return Err("Unsupported format");
        };

        let buffer_duration = Duration::from_secs_f32(1.0);
        let buffer = Arc::new(Mutex::new(AudioBuffer::new(
            sample_rate,
            channels,
            buffer_duration,
        )));
        let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(spectrum_width)));
        let mut streamer = AudioStreamer {
            buffer: buffer.clone(),
            analyzer: analyzer.clone(),
        };
        let Ok(stream) = device.build_input_stream(
            &config.into(),
            move |data, info: &_| streamer.data_callback(data.to_vec(), info),
            move |err| eprintln!("A error occured on stream: {}", err),
            None,
        ) else {
            return Err("Failed to build in/out stream");
        };

        Ok(AudioHost {
            host_id,
            sample_rate,
            channels,
            stream,
            buffer,
            spectrum_width,
            buffer_duration,
            buffer_duration_in_samples: (sample_rate as f32 * buffer_duration.as_secs_f32())
                as usize,
            analyzer,
        })
    }

    pub fn start(&self) {
        if let Err(err) = self.stream.play() {
            eprintln!("{:#?}", err);
        }
    }

    pub fn stop(&self) {
        if let Err(err) = self.stream.pause() {
            eprintln!("{:#?}", err);
        }
    }

    pub fn peek_channel(&mut self, channel: u16, duration: Duration) -> Vec<f32> {
        let data = self.buffer.lock().peek_channel(channel);
        if duration <= self.buffer_duration {
            let samples = (self.sample_rate as f32 * duration.as_secs_f32()) as usize;

            return data
                [self.buffer_duration_in_samples - samples..self.buffer_duration_in_samples]
                .to_vec();
        } else {
            return data;
        }
    }

    pub fn peek_mean_spectrum(&self) -> Vec<f32> {
        self.analyzer.lock().get_mean_spectrum()
    }
    pub fn peek_spectrum(&self) -> Vec<f32> {
        self.analyzer.lock().get_spectrum()
    }

    pub fn set_averaging_constant(&mut self, averaging_constant: f32) {
        self.analyzer
            .lock()
            .set_averaging_constant(averaging_constant);
    }

    pub fn get_averaging_constant(&self) -> f32 {
        self.analyzer.lock().get_averaging_constant()
    }

    pub fn set_spectrum_width(&mut self, fft_length: usize) {
        self.spectrum_width = fft_length;
        self.analyzer.lock().set_spectrum_width(fft_length);
    }
}
