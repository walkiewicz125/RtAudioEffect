use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::audio::{AudioBuffer, AudioStreamConsumer, StreamParameters};

use super::{ManyChannelsSpectrums, Spectrum, SpectrumAnalyzer};

pub struct AnalyzerParameters {
    spectrum_width: usize,
    refresh_time_in_samples: usize,
    number_of_spectrums_in_history: usize,
}

pub struct Spectrogram {
    parameters: Arc<AnalyzerParameters>,
    spectrum_history: Vec<Vec<Spectrum>>,
}
impl Spectrogram {
    fn new(
        parameters: Arc<AnalyzerParameters>,
        stream_parameters: Arc<StreamParameters>,
    ) -> Spectrogram {
        let empty_spectrum = vec![0.0; parameters.spectrum_width];
        let empty_spectogram_one_channel =
            vec![empty_spectrum; parameters.number_of_spectrums_in_history];
        let empty_spectogram_all_channels =
            vec![empty_spectogram_one_channel; stream_parameters.channels as usize];
        Spectrogram {
            parameters,
            spectrum_history: empty_spectogram_all_channels,
        }
    }

    fn push_spectrums(&mut self, spectrums: ManyChannelsSpectrums) {
        self.spectrum_history.push(spectrums);

        if self.spectrum_history.len() > self.parameters.number_of_spectrums_in_history {
            let oversize =
                self.spectrum_history.len() - self.parameters.number_of_spectrums_in_history;
            self.spectrum_history.drain(0..oversize);
        }
    }
}

pub struct StreamAnalyzer {
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_analyzer: SpectrumAnalyzer,
    spectrogram: Spectrogram,
}

impl AudioStreamConsumer for StreamAnalyzer {
    fn process_new_samples(&mut self, audio_buffer: Arc<Mutex<AudioBuffer>>) {
        let mut buffer = audio_buffer.lock().unwrap();

        if buffer.get_new_samples_count() >= self.analyzer_parameters.refresh_time_in_samples {
            while let Ok(new_channels_samples) = buffer.read_new_samples(
                self.analyzer_parameters.refresh_time_in_samples,
                self.analyzer_parameters.spectrum_width,
            ) {
                let mut spectrums = vec![];
                for (channel, samples) in new_channels_samples.iter().enumerate() {
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
}
