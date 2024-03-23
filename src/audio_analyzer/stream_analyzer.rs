use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, error, trace};

use crate::audio::{AudioBuffer, AudioStreamConsumer, StreamParameters};

use super::{AnalyzerParameters, ManyChannelsSpectrums, Spectrogram, SpectrumAnalyzer};

pub struct StreamAnalyzer {
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_analyzer: SpectrumAnalyzer,
    spectrogram: Spectrogram,
}

impl AudioStreamConsumer for StreamAnalyzer {
    fn process_new_samples(&mut self, audio_buffer: Arc<Mutex<AudioBuffer>>) {
        debug!("Processing new samples");
        let mut buffer = audio_buffer.lock().unwrap();

        let total_sample_count = self.analyzer_parameters.spectrum_width;
        let new_samples = self.analyzer_parameters.refresh_time_in_samples;

        while buffer.get_new_samples_count() >= new_samples {
            if let Ok(new_multichannel_samples) =
                buffer.read_new_samples(new_samples, total_sample_count)
            {
                debug!(
                    "Reading {} samples for all channels, with new samples: {}",
                    total_sample_count, new_samples
                );
                let mut spectrums = vec![];
                for (channel, samples) in new_multichannel_samples.iter().enumerate() {
                    debug!("Processing samples for channel: {}", channel);
                    spectrums.push(self.spectrum_analyzer.analyze(&samples));
                }
                self.spectrogram.push_spectrums(spectrums);
            }
        }
    }
}

impl StreamAnalyzer {
    pub fn new(
        refresh_time: Duration,
        history_duration: Duration,
        spectrum_width: usize,
        stream_parameters: Arc<StreamParameters>,
    ) -> StreamAnalyzer {
        let refresh_time_in_samples =
            (stream_parameters.sample_rate as f32 * refresh_time.as_secs_f32()) as usize;

        let number_of_spectrums_in_history =
            (history_duration.as_secs_f32() / refresh_time.as_secs_f32()) as usize;

        let parameters = Arc::new(AnalyzerParameters {
            spectrum_width,
            refresh_time_in_samples,
            number_of_spectrums_in_history,
        });

        StreamAnalyzer {
            analyzer_parameters: parameters.clone(),
            spectrum_analyzer: SpectrumAnalyzer::new(spectrum_width),
            spectrogram: Spectrogram::new(parameters, stream_parameters.clone()),
        }
    }

    pub fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters> {
        self.analyzer_parameters.clone()
    }

    pub fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.spectrogram.get_latest_spectrum()
    }
}
