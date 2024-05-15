use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{info, trace};

use crate::audio::{AudioBuffer, AudioStreamConsumer, StreamParameters};

use super::{
    AnalyzerParameters, Magnitude, ManyChannelsSpectrums, MultiChannel, Spectrogram,
    SpectrumAnalyzer, TimeSeries,
};

pub struct StreamAnalyzer {
    audio_buffer: Arc<Mutex<AudioBuffer>>,
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_analyzer: SpectrumAnalyzer,
    spectrogram: Spectrogram,
    is_alive: bool,
}

pub trait AudioAnalyzysProvider {
    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters>;
    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums;
    fn get_magnitude_timeline(&self) -> &TimeSeries<MultiChannel<Magnitude>>;
    fn get_spectrogram_for_channel(&self, channel: usize) -> (TimeSeries<Magnitude>, (u32, u32));
}

impl AudioStreamConsumer for StreamAnalyzer {
    fn process_new_samples(&mut self) {
        let total_sample_count = self.analyzer_parameters.spectrum_width;
        let new_samples = self.analyzer_parameters.refresh_time_in_samples;

        while self.audio_buffer.lock().unwrap().get_new_samples_count() >= new_samples {
            if let Ok(new_multichannel_samples) = self
                .audio_buffer
                .lock()
                .unwrap()
                .read_new_samples(new_samples, total_sample_count)
            {
                trace!(
                    "Reading {} samples for all channels, with new samples: {}",
                    total_sample_count,
                    new_samples
                );
                let mut spectrums = vec![];
                for (channel, samples) in new_multichannel_samples.iter().enumerate() {
                    trace!("Processing samples for channel: {}", channel);
                    spectrums.push(self.spectrum_analyzer.analyze(&samples));
                }

                let ps_rms = self.spectrum_analyzer.get_ps_rms(&spectrums);
                self.spectrogram.push_spectrums(spectrums);
                self.spectrogram.push_power_spectrum_rms(ps_rms);
            }
        }
    }

    fn get_audio_buffer(&self) -> Arc<Mutex<AudioBuffer>> {
        self.audio_buffer.clone()
    }

    fn get_name(&self) -> String {
        String::from("Stream spectrum analyzer")
    }
}

impl AudioAnalyzysProvider for StreamAnalyzer {
    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters> {
        self.get_analyzer_parameters()
    }

    fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.get_latest_spectrum()
    }

    fn get_magnitude_timeline(&self) -> &TimeSeries<MultiChannel<Magnitude>> {
        self.spectrogram.get_total_energy()
    }

    fn get_spectrogram_for_channel(&self, channel: usize) -> (TimeSeries<Magnitude>, (u32, u32)) {
        self.spectrogram.get_spectrogram_for_channel(channel)
    }
}

impl StreamAnalyzer {
    pub fn new(
        refresh_time: Duration,
        buffer_duration: Duration,
        spectrum_width: usize,
        stream_parameters: Arc<StreamParameters>,
    ) -> StreamAnalyzer {
        info!("Creating new StreamAnalyzer with: {stream_parameters}");

        let refresh_time_in_samples =
            (stream_parameters.sample_rate as f32 * refresh_time.as_secs_f32()) as usize;

        let number_of_spectrums_in_history =
            (buffer_duration.as_secs_f32() / refresh_time.as_secs_f32()) as usize;

        let parameters = Arc::new(AnalyzerParameters {
            spectrum_width,
            refresh_time_in_samples,
            length_of_history: number_of_spectrums_in_history,
            refresh_time,
            spectrogram_duration: buffer_duration,
            sample_rate: stream_parameters.sample_rate,
        });

        StreamAnalyzer {
            audio_buffer: Arc::new(Mutex::new(AudioBuffer::new(
                stream_parameters.clone(),
                buffer_duration,
            ))),
            analyzer_parameters: parameters.clone(),
            spectrum_analyzer: SpectrumAnalyzer::new(
                spectrum_width,
                stream_parameters.sample_rate as usize,
            ),
            spectrogram: Spectrogram::new(parameters, stream_parameters.clone()),
            is_alive: true,
        }
    }

    pub fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters> {
        self.analyzer_parameters.clone()
    }

    pub fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.spectrogram.get_latest_spectrum()
    }

    pub fn kill(&mut self) {
        self.is_alive = false;
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }
}
