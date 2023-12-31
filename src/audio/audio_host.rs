use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId, InputCallbackInfo, Stream,
};
use egui::mutex::Mutex;

struct AudioStreamer {
    buffer: Arc<Mutex<Vec<f32>>>,
}
impl AudioStreamer {
    fn data_callback(&mut self, mut data: Vec<f32>, callback_info: &InputCallbackInfo) {
        self.buffer.lock().append(&mut data);
    }
}

pub struct AudioHost {
    host_id: HostId,
    sample_rate: u32,
    channels: u16,
    stream: Stream,
    buffer: Arc<Mutex<Vec<f32>>>,
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

        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
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

    fn read_data(&mut self) -> Vec<f32> {
        let data = self.buffer.lock().clone();
        self.buffer.lock().clear();
        data
    }
}

pub struct AudioSource {
    host: AudioHost,
    left_channel_buffer: Vec<f32>,
    right_channel_buffer: Vec<f32>,
}

impl AudioSource {
    pub fn new_default_loopback() -> Result<AudioSource, String> {
        let available_hosts = cpal::available_hosts();

        if available_hosts.is_empty() {
            return Err(String::from("No host devices found"));
        }

        match AudioHost::new(available_hosts[0]) {
            Ok(host) => Ok(AudioSource {
                host,
                left_channel_buffer: vec![],
                right_channel_buffer: vec![],
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

    pub fn get_last_left_channel(&mut self, sample_count: i32) -> Vec<f32> {
        let data = self.host.read_data();
        for i in 0..data.len() {
            if i % 2 == 0 {
                self.left_channel_buffer.push(data[i]);
            } else {
                self.right_channel_buffer.push(data[i]);
            }
        }

        if self.left_channel_buffer.len() > sample_count as usize {
            self.left_channel_buffer = self
                .left_channel_buffer
                .iter()
                .rev()
                .take(sample_count as usize)
                .map(|x| x.clone())
                .collect();
        }
        if self.right_channel_buffer.len() > sample_count as usize {
            self.right_channel_buffer = self
                .right_channel_buffer
                .iter()
                .rev()
                .take(sample_count as usize)
                .map(|x| x.clone())
                .collect();
        }
        self.left_channel_buffer.clone()
    }
}
