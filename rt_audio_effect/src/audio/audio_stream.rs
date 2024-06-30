use std::sync::{Arc, Mutex};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream,
};
use log::{error, info, trace};

use super::{AudioBuffer, AudioStreamConsumer, StreamParameters};

struct AudioStreamSender {
    data_stream_receivers: Vec<Arc<Mutex<AudioBuffer>>>,
}

impl AudioStreamSender {
    pub fn new() -> Self {
        AudioStreamSender {
            data_stream_receivers: Vec::new(),
        }
    }

    pub fn add_stream_receiver(&mut self, stream_receiver: Arc<Mutex<dyn AudioStreamConsumer>>) {
        info!(
            "Adding new stream receiver: {}",
            stream_receiver.lock().unwrap().get_name()
        );
        self.data_stream_receivers
            .push(stream_receiver.lock().unwrap().get_audio_buffer());
    }

    pub fn send_data(&mut self, data: Vec<f32>) {
        for data_stream_receiver in self.data_stream_receivers.iter() {
            data_stream_receiver
                .lock()
                .unwrap()
                .store(data.clone().into());
        }
    }
}

pub struct AudioStream {
    _device: Device,
    stream: Stream,
    parameters: Arc<StreamParameters>,
    stream_sender: Arc<Mutex<AudioStreamSender>>,
}

impl AudioStream {
    pub fn new(device: Device) -> Result<AudioStream, &'static str> {
        let Ok(config) = device.default_output_config() else {
            return Err("Failed to get default config");
        };

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        let parameters = Arc::new(StreamParameters {
            sample_rate,
            channels,
        });

        info!("Creating new audio stream with: {parameters}");

        let cpal::SampleFormat::F32 = config.sample_format() else {
            error!("Unsupported format");
            panic!("Unsupported format");
        };

        let stream_sender = Arc::new(Mutex::new(AudioStreamSender::new()));
        let stream_sender_copy = stream_sender.clone();
        let Ok(stream) = device.build_input_stream(
            &config.into(),
            move |data, _info: &InputCallbackInfo| {
                trace!(target:"cpal::Stream", "Sending new data with len: {}", data.len());
                stream_sender_copy.lock().unwrap().send_data(data.to_vec());
            },
            move |err| error!("A error occured on stream: {err:?}"),
            None,
        ) else {
            error!("Failed to build in/out stream");
            panic!("Failed to build in/out stream");
        };

        Ok(AudioStream {
            _device: device,
            stream,
            parameters,
            stream_sender,
        })
    }

    pub fn start(&self) {
        info!("Starting stream");

        if let Err(err) = self.stream.play() {
            error!("Error occured during start(): {err:#?}");
        }
    }

    pub fn stop(&self) {
        info!("Stopping stream");

        if let Err(err) = self.stream.pause() {
            error!("Error occured during stop(): {err:#?}");
        }
    }

    pub fn get_parameters(&self) -> Arc<StreamParameters> {
        self.parameters.clone()
    }

    pub fn add_stream_consumer(&mut self, stream_consumer: Arc<Mutex<dyn AudioStreamConsumer>>) {
        self.stream_sender
            .lock()
            .unwrap()
            .add_stream_receiver(stream_consumer)
    }
}
