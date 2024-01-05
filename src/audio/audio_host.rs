use std::{ops::Deref, sync::Arc, time::Duration};

use super::{audio_buffer::AudioBuffer, stream_analyzer::StreamAnalyzer};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId, InputCallbackInfo, Stream,
};
use egui::mutex::Mutex;

struct AudioStreamer {
    buffer: Arc<Mutex<AudioBuffer>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
}
impl AudioStreamer {
    fn data_callback(&mut self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        self.buffer.lock().store(data);
        self.analyzer.lock().analyze(&mut self.buffer.lock());
    }
}

pub struct AudioHost {
    host_id: HostId,
    sample_rate: u32,
    channels: u16,
    stream: Stream,
    buffer: Arc<Mutex<AudioBuffer>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,

    spectrum_width: usize,
    buffer_duration: Duration,
    buffer_duration_in_samples: usize,
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

    fn start(&self) {
        if let Err(err) = self.stream.play() {
            eprintln!("{:#?}", err);
        }
    }

    fn stop(&self) {
        if let Err(err) = self.stream.pause() {
            eprintln!("{:#?}", err);
        }
    }

    fn peek_channel(&mut self, channel: u16, duration: Duration) -> Vec<f32> {
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

    fn peek_mean_spectrum(&self) -> Vec<f32> {
        self.analyzer.lock().get_mean_spectrum()
    }
    fn peek_spectrum(&self) -> Vec<f32> {
        self.analyzer.lock().get_spectrum()
    }

    fn set_averaging_constant(&mut self, averaging_constant: f32) {
        self.analyzer
            .lock()
            .set_averaging_constant(averaging_constant);
    }

    fn get_averaging_constant(&self) -> f32 {
        self.analyzer.lock().get_averaging_constant()
    }

    fn set_spectrum_width(&mut self, fft_length: usize) {
        self.spectrum_width = fft_length;
        self.analyzer.lock().set_spectrum_width(fft_length);
    }
}

pub struct AudioAnalyzysSource {
    host: AudioHost,
    spectrum_width: usize,
}

impl AudioAnalyzysSource {
    pub fn new_default_loopback(spectrum_width: usize) -> Result<AudioAnalyzysSource, String> {
        let available_hosts = cpal::available_hosts();

        if available_hosts.is_empty() {
            return Err(String::from("No host devices found"));
        }

        match AudioHost::new(available_hosts[0], spectrum_width) {
            Ok(host) => Ok(AudioAnalyzysSource {
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
}
