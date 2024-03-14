pub trait AudioStreamConsumer {
    fn process_new_samples(&mut self, audio_samples: Vec<f32>);
}
