use std::sync::Arc;

use crate::audio::StreamParameters;

use super::{AnalyzerParameters, ManyChannelsSpectrums, Spectrum};

type Magnitude = f32;
type MultiChannel<T> = Vec<T>;
type TimeSeries<T> = Vec<T>;

pub struct Spectrogram {
    stream_parameters: Arc<StreamParameters>,
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_history: TimeSeries<MultiChannel<Spectrum>>,
    peek_magnitude: TimeSeries<MultiChannel<Magnitude>>,
    rms_magnitude: TimeSeries<MultiChannel<Magnitude>>,
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
            peek_magnitude: Self::create_time_series::<Magnitude>(
                &stream_parameters,
                &analyzer_parameters,
                0.0,
            ),
            rms_magnitude: Self::create_time_series::<Magnitude>(
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
        let mut rms_all_channels: MultiChannel<Magnitude> = vec![];
        let mut peek_all_channels: MultiChannel<Magnitude> = vec![];

        for spectrum in &spectrums {
            let mut peek_magnitude: Magnitude = 0.0;
            let mut rms_magnitude: Magnitude = 0.0;
            for value in spectrum {
                peek_magnitude = peek_magnitude.max(*value);
                rms_magnitude += value.powi(2);
            }
            rms_magnitude = rms_magnitude.sqrt() / spectrum.len() as f32;

            peek_all_channels.push(peek_magnitude);
            rms_all_channels.push(rms_magnitude);
        }

        self.spectrum_history.push(spectrums);

        Self::trimm_time_series(
            &mut self.peek_magnitude,
            self.analyzer_parameters.length_of_history,
        );
        Self::trimm_time_series(
            &mut self.rms_magnitude,
            self.analyzer_parameters.length_of_history,
        );
        Self::trimm_time_series(
            &mut self.spectrum_history,
            self.analyzer_parameters.length_of_history,
        );
    }

    pub fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        self.spectrum_history.last().unwrap().clone()
    }
}
