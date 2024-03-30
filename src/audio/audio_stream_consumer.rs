use std::sync::{Arc, Mutex};

use super::AudioBuffer;

pub trait AudioStreamConsumer: Send {
    fn process_new_samples(&mut self);
    fn get_audio_buffer(&self) -> Arc<Mutex<AudioBuffer>>;
}
