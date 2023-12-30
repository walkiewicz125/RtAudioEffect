use std::sync::mpsc::{self, Receiver, Sender};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId, InputCallbackInfo, Stream,
};

struct AudioStreamer {
    tx_channel: Sender<Vec<f32>>,
}
impl AudioStreamer {
    fn data_callback(&self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        match self.tx_channel.send(data) {
            Ok(_) => println!(
                "Succeeded to send data. Timestamp: {:#?}",
                callback_info.timestamp()
            ),
            Err(_) => {
                eprintln!("Failed to send data")
            }
        }
    }
}

struct AudioHost {
    host_id: HostId,
    sample_rate: u32,
    channels: u16,
    stream: Stream,
    rx_channel: Receiver<Vec<f32>>,
}

impl AudioHost {
    fn new(host_id: HostId) -> Result<AudioHost, &'static str> {
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

        let (tx_channel, rx_channel) = mpsc::channel::<Vec<f32>>();
        let streamer = AudioStreamer { tx_channel };
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
            rx_channel,
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
}
