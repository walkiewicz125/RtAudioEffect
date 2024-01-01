use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId, InputCallbackInfo, Stream,
};
use egui::mutex::Mutex;

struct AudioBuffer {
    sample_rate: u32,
    channels: u16,
    buffer_duration: Duration,
    buffer_duration_in_samples: usize,

    buffers: Vec<Vec<f32>>,
}

impl AudioBuffer {
    fn new(sample_rate: u32, channels: u16, buffer_duration: Duration) -> AudioBuffer {
        let buffer_duration_in_samples: usize =
            (sample_rate as f32 * buffer_duration.as_secs_f32()) as usize;
        AudioBuffer {
            sample_rate,
            channels,
            buffer_duration,
            buffer_duration_in_samples,
            buffers: vec![vec![0.0; buffer_duration_in_samples]; channels as usize],
        }
    }

    fn store(&mut self, data: Vec<f32>) {
        for i in 0..data.len() {
            let channel = i % self.channels as usize;
            self.buffers[channel].push(data[i]);
        }

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

    fn get_channel_last(&self, channel: u16, duration: Duration) -> Vec<f32> {
        if duration <= self.buffer_duration {
            let samples = (self.sample_rate as f32 * duration.as_secs_f32()) as usize;

            return self.buffers[channel as usize]
                [self.buffer_duration_in_samples - samples..self.buffer_duration_in_samples]
                .to_vec();
        } else {
            return self.buffers[channel as usize].to_vec();
        }
    }
}

struct AudioStreamer {
    buffer: Arc<Mutex<AudioBuffer>>,
}
impl AudioStreamer {
    fn data_callback(&mut self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        self.buffer.lock().store(data);
    }
}

pub struct AudioHost {
    host_id: HostId,
    sample_rate: u32,
    channels: u16,
    stream: Stream,
    buffer: Arc<Mutex<AudioBuffer>>,
}

impl AudioHost {
    pub fn new(host_id: HostId) -> Result<AudioHost, &'static str> {
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

        let buffer = Arc::new(Mutex::new(AudioBuffer::new(
            sample_rate,
            channels,
            Duration::from_secs_f32(1.0),
        )));
        let mut streamer = AudioStreamer {
            buffer: buffer.clone(),
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

    fn read_data(&mut self, duration: Duration) -> Vec<f32> {
        let guard = self.buffer.lock();
        let data = guard.get_channel_last(0, duration);
        data
    }
}

pub struct AudioSource {
    host: AudioHost,
}

impl AudioSource {
    pub fn new_default_loopback() -> Result<AudioSource, String> {
        let available_hosts = cpal::available_hosts();

        if available_hosts.is_empty() {
            return Err(String::from("No host devices found"));
        }

        match AudioHost::new(available_hosts[0]) {
            Ok(host) => Ok(AudioSource { host }),
            Err(err) => return Err(String::from(err)),
        }
    }
    pub fn start(&self) {
        self.host.start();
    }

    pub fn stop(&self) {
        self.host.stop();
    }

    pub fn get_last_left_channel(&mut self, sample_count: i32) -> Vec<f32> {
        let duration = Duration::from_secs_f32(sample_count as f32 / self.host.sample_rate as f32);
        self.host.read_data(duration)
    }
}
