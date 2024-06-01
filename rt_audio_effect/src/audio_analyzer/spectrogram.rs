use std::sync::Arc;

use crate::audio::StreamParameters;

use super::{AnalyzerParameters, MultiChannel, Spectrum, TimeSeries};

pub type Magnitude = f32;

pub struct Spectrogram {
    stream_parameters: Arc<StreamParameters>,
    spectrum_history: MultiChannel<TimeSeries<Magnitude>>,
}

impl Spectrogram {
    pub fn new(
        analyzer_parameters: Arc<AnalyzerParameters>,
        stream_parameters: Arc<StreamParameters>,
    ) -> Spectrogram {
        Spectrogram {
            stream_parameters: stream_parameters.clone(),
            spectrum_history: MultiChannel::new(
                stream_parameters.channels as usize,
                TimeSeries::new(
                    analyzer_parameters.length_of_history,
                    analyzer_parameters.spectrum_width / 2,
                    0.0,
                ),
            ),
        }
    }

    pub fn push_spectrums(&mut self, spectrums: MultiChannel<Spectrum>) {
        assert!(spectrums.len() == self.stream_parameters.channels as usize);
        assert!(spectrums.len() == self.spectrum_history.len());

        spectrums.into_iter().enumerate().for_each(|(i, data)| {
            self.spectrum_history.channels[i].push(data.into());
        });
    }

    pub fn get_latest_spectrum(&self) -> MultiChannel<Spectrum> {
        let mut spectrums: Vec<Spectrum> = vec![];
        self.spectrum_history.channels.iter().for_each(|channel| {
            spectrums.push(channel.get_last().into());
        });
        spectrums.into()
    }

    pub fn get_spectrogram_for_channel(
        &self,
        channel: usize,
    ) -> (TimeSeries<Magnitude>, (u32, u32)) {
        let x_count = self.spectrum_history.channels[channel].get_width();
        let y_count = self.spectrum_history.channels[channel].get_length();
        (
            self.spectrum_history.channels[channel].clone(),
            (x_count as u32, y_count as u32),
        )
    }
}
