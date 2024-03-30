use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream,
};
use log::{debug, error, info, trace};

use super::{AudioBuffer, AudioStreamConsumer, MixedChannelsSamples, StreamParameters};

struct StreamConsumerHandler {
    audio_buffer: Arc<Mutex<AudioBuffer>>,
    stream_consumer: Arc<Mutex<dyn AudioStreamConsumer>>,
    consumer_name: String,
}

pub struct AudioDevice {
    _device: Device,
    stream: Stream,
    parameters: Arc<StreamParameters>,
    data_receiver: Receiver<MixedChannelsSamples>,
    consumers_handlers: Vec<StreamConsumerHandler>,
    last_update_time: Instant,
}

impl AudioDevice {
    pub fn new(device: Device) -> Result<AudioDevice, &'static str> {
        let Ok(config) = device.default_output_config() else {
            return Err("Failed to get default config");
        };

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        info!("Sample rate: {sample_rate}, channels: {channels}");

        let cpal::SampleFormat::F32 = config.sample_format() else {
            error!("Unsupported format");
            panic!("Unsupported format");
        };

        let parameters = Arc::new(StreamParameters {
            sample_rate,
            channels,
        });

        let (data_sender, data_receiver) = mpsc::channel::<MixedChannelsSamples>();

        let Ok(stream) = device.build_input_stream(
            &config.into(),
            move |data, _info: &InputCallbackInfo| {
                trace!(target:"cpal::Stream", "Sending new data with len: {}", data.len());
                data_sender.send(data.to_vec()).unwrap();
            },
            move |err| error!("A error occured on stream: {err:?}"),
            None,
        ) else {
            error!("Failed to build in/out stream");
            panic!("Failed to build in/out stream");
        };

        Ok(AudioDevice {
            _device: device,
            stream,
            parameters,
            data_receiver,
            consumers_handlers: vec![],
            last_update_time: Instant::now(),
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

    pub fn add_stream_consumer(
        &mut self,
        stream_consumer: Arc<Mutex<dyn AudioStreamConsumer>>,
        consumer_name: Option<String>,
    ) {
        let buffer_duration = Duration::from_secs_f32(1.0);
        let buffer_duration_in_samples: usize =
            (self.parameters.sample_rate as f32 * buffer_duration.as_secs_f32()) as usize;

        let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(
            self.parameters.clone(),
            buffer_duration_in_samples,
        )));
        let handler = StreamConsumerHandler {
            audio_buffer,
            stream_consumer,
            consumer_name: consumer_name.unwrap_or(String::from("unnamed")),
        };

        info!("Adding new stream consumer: {}", handler.consumer_name);
        self.consumers_handlers.push(handler);
    }

    pub fn update(&mut self) {
        let update_time = Instant::now();
        let update_period = self.last_update_time.elapsed();
        let mut data_packets = 0;
        while let Ok(new_data) = self.data_receiver.recv_timeout(Duration::from_secs(0)) {
            data_packets += 1;
            trace!("");
            trace!("Receiving new data with len: {}", new_data.len());

            for handler in &self.consumers_handlers {
                trace!(
                    "Calling processing in consumer \"{}\"",
                    handler.consumer_name
                );
                handler.audio_buffer.lock().unwrap().store(new_data.clone());
                handler
                    .stream_consumer
                    .lock()
                    .unwrap()
                    .process_new_samples(handler.audio_buffer.clone());
            }
        }

        if data_packets > 0 {
            let processing_time = update_time.elapsed();
            self.last_update_time = update_time;

            debug!(
                "Update thread. Processing time: {}, Data packets {}",
                processing_time.as_secs_f32() / update_period.as_secs_f32(),
                data_packets
            );
        }
    }

    pub fn get_parameters(&self) -> Arc<StreamParameters> {
        self.parameters.clone()
    }
}
