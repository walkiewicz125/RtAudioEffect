use std::sync::{Arc, Mutex};

use cpal::InputCallbackInfo;
use log::{debug, trace};

use super::AudioBuffer;

pub struct AudioStreamReceiver {
    sample_buffer: Arc<Mutex<AudioBuffer>>,
}

impl AudioStreamReceiver {
    pub fn new(sample_buffer: Arc<Mutex<AudioBuffer>>) -> AudioStreamReceiver {
        AudioStreamReceiver { sample_buffer }
    }
    pub fn data_callback(&mut self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        trace!("Callback info: {callback_info:#?}");
        self.sample_buffer.lock().unwrap().store(data);
    }
}
