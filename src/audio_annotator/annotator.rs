use std::{sync::Arc, time::Duration};

use log::{debug, info};

use crate::audio_analyzer::{
    AnalyzerParameters, ManyChannelsSpectrums, MultiChannel, StreamAnalyzerReceiver,
};

pub struct StreamAnalyzerAnnotator {
    parameters: Arc<AnalyzerParameters>,
    ranges_indices: Vec<Vec<usize>>,
    duration: Duration,
    filters: MultiChannel<Vec<SpectrumFilter>>,
    ind: usize,
}

/*
Sub-Bass: 20 to 60 Hz
Bass: 60 to 250 Hz
Low Midrange: 250 to 500 Hz
Midrange: 500 Hz to 2 kHz
Upper Midrange: 2 to 4 kHz
Treble: 4 kHz to 6 kHz
Upper Treble: 6 to 20 kHz
*/

impl StreamAnalyzerAnnotator {
    const FREQUENCY_RANGES: [[usize; 2]; 7] = [
        [0, 60],
        [60, 250],
        [250, 500],
        [500, 2000],
        [2000, 4000],
        [4000, 6000],
        [6000, 20000],
    ];

    pub fn new(parameters: Arc<AnalyzerParameters>, duration: Duration, channels: usize) -> Self {
        let bin_width = parameters.sample_rate as f32 / parameters.spectrum_width as f32;

        let ranges_indices: Vec<Vec<usize>> = Self::FREQUENCY_RANGES
            .iter()
            .map(|range| {
                let start_bin = (range[0] as f32 / bin_width).round() as usize;
                let end_bin = (range[1] as f32 / bin_width).round() as usize;
                (start_bin..end_bin).collect()
            })
            .collect();

        debug!("frequency ranges indices [{:?}]", ranges_indices.len());

        ranges_indices.iter().enumerate().for_each(|(ind, range)| {
            debug!(
                "Range: {}, indices count: {}, first: {}, last: {}",
                ind,
                range.len(),
                range[0],
                range[range.len() - 1]
            );
        });

        let channel_filters: Vec<SpectrumFilter> = ranges_indices
            .iter()
            .map(|range| {
                SpectrumFilter::new(range.to_vec(), duration, parameters.spectrogram_duration)
            })
            .collect();

        let filters = MultiChannel::new(channels, channel_filters);

        Self {
            parameters,
            ranges_indices,
            duration,
            filters,
            ind: 0,
        }
    }

    pub fn push_spectrum(&mut self, spectrums: &ManyChannelsSpectrums) {
        info!("Pushing spectrum to annotator");
        spectrums
            .iter()
            .enumerate()
            .for_each(|(channel, spectrum)| {
                self.filters.channels[channel]
                    .iter_mut()
                    .for_each(|filter| {
                        filter.filter(spectrum);
                    });
            });
    }
}

impl StreamAnalyzerReceiver for StreamAnalyzerAnnotator {
    fn receive(&mut self, spectrums: &ManyChannelsSpectrums) {
        self.push_spectrum(spectrums);
    }
}

#[derive(Clone)]
struct HistoryBuffer<T> {
    duration: Duration,
    entry_time: Duration,
    duration_length: usize,
    history: Vec<T>,
}

impl<T> HistoryBuffer<T>
where
    T: Clone,
{
    pub fn new(duration: Duration, entry_time: Duration) -> Self {
        let duration_length = (duration.as_secs_f32() / entry_time.as_secs_f32()).round() as usize;

        Self {
            duration,
            entry_time,
            duration_length,
            history: Vec::with_capacity(duration_length),
        }
    }

    pub fn push(&mut self, value: T) {
        self.history.push(value);
        if self.history.len() > self.duration_length {
            self.history.remove(0);
        }
    }

    pub fn get(&self) -> Vec<T> {
        self.history.clone()
    }

    pub fn peek(&self) -> &[T] {
        self.history.as_slice()
    }
}

#[derive(Clone)]
struct SpectrumFilter {
    indices: Vec<usize>,
    history: HistoryBuffer<f32>,
}

impl SpectrumFilter {
    pub fn new(indices: Vec<usize>, duration: Duration, spectrum_duration: Duration) -> Self {
        Self {
            indices,
            history: HistoryBuffer::new(duration, spectrum_duration),
        }
    }

    pub fn filter(&mut self, spectrum: &[f32]) {
        let filtered_spectrum = self.indices.iter().map(|&ind| spectrum[ind]).sum();
        self.history.push(filtered_spectrum);

        info!(
            "Filtered spectrum: {:?}",
            self.history.peek().last().unwrap()
        );
    }
}
