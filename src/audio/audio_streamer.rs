use cpal::InputCallbackInfo;
use egui::mutex::Mutex;
use std::sync::Arc;

use super::{AudioBuffer, StreamAnalyzer};

pub struct AudioStreamer {
    pub buffer: Arc<Mutex<AudioBuffer>>,
    pub analyzer: Arc<Mutex<StreamAnalyzer>>,
}

impl AudioStreamer {
    pub fn data_callback(&mut self, data: Vec<f32>, callback_info: &InputCallbackInfo) {
        self.buffer.lock().store(data);
        self.analyzer.lock().analyze(&mut self.buffer.lock());
    }
}
