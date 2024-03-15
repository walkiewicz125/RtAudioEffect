use std::{sync::Arc, time::Duration};

use log::debug;
use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::audio::{AudioBuffer, AudioStreamConsumer, StreamParameters};

use super::SpectrumAnalyzer;
type Spectrum = Vec<f32>;

pub struct AnalyzerParameters {
    spectrum_width: usize,
    refresh_time_in_samples: usize,
    number_of_spectrums_in_history: usize,
}

pub struct Spectrogram {
    parameters: Arc<AnalyzerParameters>,
    stream_parameters: Arc<StreamParameters>,
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
            stream_parameters,
            spectrum_history: empty_spectogram_all_channels,
        }
    }
}

pub struct StreamAnalyzer {
    analyzer_parameters: Arc<AnalyzerParameters>,
    stream_parameters: Arc<StreamParameters>,
    spectrum_analyzer: SpectrumAnalyzer,
    spectrogram: Spectrogram,
    buffer: AudioBuffer,
}

impl AudioStreamConsumer for StreamAnalyzer {
    fn process_new_samples(&mut self, channels_audio_samples: Vec<f32>) {
        self.buffer.store(channels_audio_samples);
        debug!("eeeeeeeeeee");

        if self.buffer.get_new_samples_count() >= self.analyzer_parameters.refresh_time_in_samples {
            while let Ok(new_channels_samples) = self.buffer.read_new_samples(
                self.analyzer_parameters.refresh_time_in_samples,
                self.analyzer_parameters.spectrum_width,
            ) {
                for (channel, samples) in new_channels_samples.iter().enumerate() {
                    self.spectrum_analyzer.analyze(&samples);
                    debug!("ADADADADAD");
                }
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
            stream_parameters: stream_parameters.clone(),
            spectrum_analyzer: SpectrumAnalyzer::new(spectrum_width),
            spectrogram: Spectrogram::new(parameters, stream_parameters.clone()),
            buffer: AudioBuffer::new(
                stream_parameters.clone(),
                stream_parameters.sample_rate as usize,
            ),
        }
    }
}
