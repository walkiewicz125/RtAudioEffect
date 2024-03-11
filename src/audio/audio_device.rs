use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, Stream,
};
use log::{error, info, trace};

use crate::audio::AudioStreamReceiver;

use super::{AudioBuffer, AudioDeviceParameters};

pub trait AudioDataConsumer {
    fn consume_samples(&mut self, channels_samples: Vec<Vec<f32>>);
}

pub struct AudioDevice {
    device: Device,
    stream: Stream,
    parameters: AudioDeviceParameters,
    buffer_duration: Duration,
    audio_buffer: Arc<Mutex<AudioBuffer>>,
    data_receiver: Receiver<Vec<f32>>,
    data_callback: Option<Arc<Mutex<dyn AudioDataConsumer>>>,
}
pub enum Overlap {
    None,
    Half,
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

        let parameters = AudioDeviceParameters {
            sample_rate,
            channels,
        };

        let buffer_duration = Duration::from_secs_f32(1.0);
        let buffer_duration_in_samples: usize =
            (sample_rate as f32 * buffer_duration.as_secs_f32()) as usize;
        let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(
            channels,
            buffer_duration_in_samples,
        )));

        let (data_sender, data_receiver) = mpsc::channel();

        let mut stream_receiver = AudioStreamReceiver::new(data_sender);

        let Ok(stream) = device.build_input_stream(
            &config.into(),
            move |data, info: &_| stream_receiver.data_callback(data.to_vec(), info),
            move |err| error!("A error occured on stream: {err:?}"),
            None,
        ) else {
            error!("Failed to build in/out stream");
            panic!("Failed to build in/out stream");
        };

        Ok(AudioDevice {
            device,
            stream,
            parameters,
            buffer_duration,
            audio_buffer,
            data_receiver,
            data_callback: None,
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

    pub fn set_callback(
        &mut self,
        update_duration: Duration,
        overlap: Overlap,
        data_consumer: Arc<Mutex<dyn AudioDataConsumer>>,
    ) {
        self.data_callback = Some(data_consumer);
    }

    pub fn run(&mut self) {
        while let Ok(new_data) = self.data_receiver.recv_timeout(Duration::from_secs(0)) {
            self.audio_buffer.lock().unwrap().store(new_data);

            let new_samples = self.audio_buffer.lock().unwrap().get_new_samples_count();

            let samples = self
                .audio_buffer
                .lock()
                .unwrap()
                .read_samples(new_samples, 0)
                .unwrap();

            if let Some(callback) = &self.data_callback {
                callback.lock().unwrap().consume_samples(samples);
            }
        }
    }
}
