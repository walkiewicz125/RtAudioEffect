use std::sync::Arc;

use crate::audio::StreamParameters;

use super::{AnalyzerParameters, ManyChannelsSpectrums, Spectrum};

pub struct Spectrogram {
    parameters: Arc<AnalyzerParameters>,
    spectrum_history: Vec<Vec<Spectrum>>,
}

impl Spectrogram {
    pub fn new(
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

    pub fn push_spectrums(&mut self, spectrums: ManyChannelsSpectrums) {
        self.spectrum_history.push(spectrums);

        if self.spectrum_history.len() > self.parameters.number_of_spectrums_in_history {
            let oversize =
                self.spectrum_history.len() - self.parameters.number_of_spectrums_in_history;
            self.spectrum_history.drain(0..oversize);
        }
    }
}
