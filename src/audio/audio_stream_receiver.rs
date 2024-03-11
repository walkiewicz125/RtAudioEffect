use std::sync::mpsc::Sender;

use cpal::InputCallbackInfo;
use log::trace;

pub struct AudioStreamReceiver {
    data_sender: Sender<Vec<f32>>,
}

impl AudioStreamReceiver {
    pub fn new(data_sender: Sender<Vec<f32>>) -> AudioStreamReceiver {
        AudioStreamReceiver { data_sender }
    }
    pub fn data_callback(&mut self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        trace!("Callback info: {callback_info:#?}");
        self.data_sender.send(data);
    }
}
