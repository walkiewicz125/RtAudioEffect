use std::sync::{Arc, Mutex};

use super::AudioBuffer;

pub trait AudioStreamConsumer: Send {
    fn process_new_samples(&mut self, audio_buffer: Arc<Mutex<AudioBuffer>>);
}
