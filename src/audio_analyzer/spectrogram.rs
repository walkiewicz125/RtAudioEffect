use std::sync::Arc;

use crate::audio::StreamParameters;

use super::{AnalyzerParameters, ManyChannelsSpectrums, Spectrum};

pub type Magnitude = f32;
pub type MultiChannel<T> = Vec<T>;
pub type TimeSeries<T> = Vec<T>;

pub struct Spectrogram {
    stream_parameters: Arc<StreamParameters>,
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_history: TimeSeries<MultiChannel<Spectrum>>,
    power_spectrum_rms: TimeSeries<MultiChannel<Magnitude>>,
}

impl Spectrogram {
    pub fn new(
        analyzer_parameters: Arc<AnalyzerParameters>,
        stream_parameters: Arc<StreamParameters>,
    ) -> Spectrogram {
        Spectrogram {
            stream_parameters: stream_parameters.clone(),
            analyzer_parameters: analyzer_parameters.clone(),
            spectrum_history: Self::create_time_series::<Spectrum>(
                &stream_parameters,
                &analyzer_parameters,
                vec![0.0; analyzer_parameters.spectrum_width],
            ),
            power_spectrum_rms: Self::create_time_series::<Magnitude>(
                &stream_parameters,
                &analyzer_parameters,
                0.0,
            ),
        }
    }

    fn create_time_series<T: Default + Clone>(
        stream_parameters: &Arc<StreamParameters>,
        analyzer_parameters: &Arc<AnalyzerParameters>,
        default_value: T,
    ) -> TimeSeries<MultiChannel<T>> {
        let default_multi_channel = vec![default_value; stream_parameters.channels as usize];
        let default_time_series =
            vec![default_multi_channel; analyzer_parameters.length_of_history];
        default_time_series
    }

    fn trimm_time_series<T>(time_series: &mut TimeSeries<MultiChannel<T>>, max_length: usize) {
        if time_series.len() > max_length {
            let oversize = time_series.len() - max_length;
            time_series.drain(0..oversize);
        }
    }

    pub fn push_spectrums(&mut self, spectrums: ManyChannelsSpectrums) {
        self.spectrum_history.push(spectrums);

        Self::trimm_time_series(
            &mut self.power_spectrum_rms,
            self.analyzer_parameters.length_of_history,
        );
        Self::trimm_time_series(
            &mut self.spectrum_history,
            self.analyzer_parameters.length_of_history,
        );
    }

    pub fn push_power_spectrum_rms(&mut self, power_spectrum_rms: MultiChannel<Magnitude>) {
        self.power_spectrum_rms.push(power_spectrum_rms);

        Self::trimm_time_series(
            &mut self.power_spectrum_rms,
            self.analyzer_parameters.length_of_history,
        );
    }

    pub fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.spectrum_history.last().unwrap().clone()
    }

    pub fn get_total_energy(&self) -> &Vec<Vec<f32>> {
        &self.power_spectrum_rms
    }
}
