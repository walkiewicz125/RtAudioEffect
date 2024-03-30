use std::sync::{mpsc::Sender, Arc};

use cpal::{
    traits::{DeviceTrait, StreamTrait},
    Device, InputCallbackInfo, Stream,
};
use log::{error, info, trace};

use super::StreamParameters;

pub struct AudioStreamSource {
    _device: Device,
    stream: Stream,
    parameters: Arc<StreamParameters>,
}

impl AudioStreamSource {
    pub fn new(
        device: Device,
        data_chanel_tx: Sender<Vec<f32>>,
    ) -> Result<AudioStreamSource, &'static str> {
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

        let Ok(stream) = device.build_input_stream(
            &config.into(),
            move |data, _info: &InputCallbackInfo| {
                trace!(target:"cpal::Stream", "Sending new data with len: {}", data.len());
                data_chanel_tx.send(data.to_vec()).unwrap();
            },
            move |err| error!("A error occured on stream: {err:?}"),
            None,
        ) else {
            error!("Failed to build in/out stream");
            panic!("Failed to build in/out stream");
        };

        Ok(AudioStreamSource {
            _device: device,
            stream,
            parameters,
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
}
