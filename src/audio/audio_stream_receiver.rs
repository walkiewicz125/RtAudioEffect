use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::{Duration, Instant},
};

use cpal::Device;
use log::{debug, info, trace};

use super::{AudioBuffer, AudioStreamConsumer, MixedChannelsSamples, StreamParameters};

struct StreamConsumerHandler {
    audio_buffer: Arc<Mutex<AudioBuffer>>,
    stream_consumer: Arc<Mutex<dyn AudioStreamConsumer>>,
    consumer_name: String,
}

pub struct AudioStreamReceiver {
    parameters: Arc<StreamParameters>,
    data_receiver: Receiver<MixedChannelsSamples>,
    consumers_handlers: Arc<Mutex<Vec<StreamConsumerHandler>>>,
    last_update_time: Instant,
}

impl AudioStreamReceiver {
    pub fn new(
        parameters: Arc<StreamParameters>,
        data_channel_rx: Receiver<Vec<f32>>,
    ) -> Result<AudioStreamReceiver, &'static str> {
        Ok(AudioStreamReceiver {
            parameters,
            data_receiver: data_channel_rx,
            consumers_handlers: Arc::new(Mutex::new(vec![])),
            last_update_time: Instant::now(),
        })
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
        self.consumers_handlers.lock().unwrap().push(handler);
    }

    pub fn update(&mut self) {
        let update_time = Instant::now();
        let update_period = self.last_update_time.elapsed();
        let mut data_packets = 0;
        while let Ok(new_data) = self.data_receiver.recv_timeout(Duration::from_secs(0)) {
            data_packets += 1;
            trace!("");
            trace!("Receiving new data with len: {}", new_data.len());

            for handler in self.consumers_handlers.lock().unwrap().iter() {
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
