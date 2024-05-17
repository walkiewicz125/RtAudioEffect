use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{debug, info};

use crate::audio_analyzer::{
    AnalyzerParameters, ManyChannelsSpectrums, MultiChannel, StreamAnalyzerReceiver,
};

// add send trait

pub struct StreamAnnotation {}

pub trait StreamAnnotationReceiver: Send {
    fn receive(&mut self, annotations: &StreamAnnotation);
}

pub struct StreamAnalyzerAnnotator {
    parameters: Arc<AnalyzerParameters>,
    ranges_indices: Vec<Vec<usize>>,
    duration: Duration,
    filters: MultiChannel<Vec<SpectrumFilter>>,
    receivers: Vec<Arc<Mutex<dyn StreamAnnotationReceiver>>>,
    bass_annotator: BasAnnotator,
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
#[derive(PartialEq, Debug, Clone, Copy)]
enum BasState {
    BasUp,
    BasDown,
}

enum BasEvent {
    NewState {
        sub_bas: Option<BasState>,
        bas: Option<BasState>,
    },
}

struct BasAnnotator {
    bas_state: BasState,
    sub_bass_state: BasState,
}

impl BasAnnotator {
    pub fn new() -> Self {
        Self {
            bas_state: BasState::BasDown,
            sub_bass_state: BasState::BasDown,
        }
    }

    fn update(&mut self, filters: &[SpectrumFilter]) -> BasEvent {
        let sub_bas_avg = filters[StreamAnalyzerAnnotator::SUB_BAS].get_average();
        let last_sub_bass = filters[StreamAnalyzerAnnotator::SUB_BAS].get_last();

        let new_sub_bas_state = if last_sub_bass > sub_bas_avg {
            BasState::BasUp
        } else {
            BasState::BasDown
        };

        let bas_avg = filters[StreamAnalyzerAnnotator::BAS].get_average();
        let last_bass = filters[StreamAnalyzerAnnotator::BAS].get_last();

        let new_bas_state = if last_bass > bas_avg {
            BasState::BasUp
        } else {
            BasState::BasDown
        };

        if new_sub_bas_state != self.sub_bass_state {
            self.sub_bass_state = new_sub_bas_state;
            info!("Sub bass state changed to: {:#?}", new_sub_bas_state)
        }

        if new_bas_state != self.bas_state {
            self.bas_state = new_bas_state;
            info!("Bass state changed to: {:#?}", new_bas_state)
        }

        BasEvent::NewState {
            sub_bas: if new_sub_bas_state != self.sub_bass_state {
                Some(new_sub_bas_state)
            } else {
                None
            },
            bas: if new_bas_state != self.bas_state {
                Some(new_bas_state)
            } else {
                None
            },
        }
    }
}

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

    const SUB_BAS: usize = 0;
    const BAS: usize = 1;
    const LOW_MIDRANGE: usize = 2;
    const MIDRANGE: usize = 3;
    const UPPER_MIDRANGE: usize = 4;
    const TREBLE: usize = 5;
    const UPPER_TREBLE: usize = 6;

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
            .map(|range| SpectrumFilter::new(range.to_vec(), duration, parameters.refresh_time))
            .collect();

        let filters = MultiChannel::new(channels, channel_filters);

        Self {
            parameters,
            ranges_indices,
            duration,
            filters,
            receivers: vec![],
            bass_annotator: BasAnnotator::new(),
        }
    }

    fn check_for_annotation(&mut self) {
        self.filters
            .channels
            .iter()
            .enumerate()
            .for_each(|(channel, filters)| {
                self.bass_annotator.update(filters);
            });
    }

    pub fn push_spectrum(&mut self, spectrums: &ManyChannelsSpectrums) {
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

        self.check_for_annotation();
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
    const SMOOTHING_FACTOR: f32 = 0.99;

    pub fn new(indices: Vec<usize>, duration: Duration, spectrum_duration: Duration) -> Self {
        Self {
            indices,
            history: HistoryBuffer::new(duration, spectrum_duration),
        }
    }

    pub fn filter(&mut self, spectrum: &[f32]) {
        let filtered_spectrum: f32 = self.indices.iter().map(|&ind| spectrum[ind]).sum();
        let last = self.history.peek().last().cloned().unwrap_or(0.0);
        let new =
            last * Self::SMOOTHING_FACTOR + filtered_spectrum * (1.0 - Self::SMOOTHING_FACTOR);
        self.history.push(new);
    }

    pub fn get_min(&self) -> f32 {
        self.history
            .peek()
            .iter()
            .cloned()
            .reduce(f32::min)
            .expect("Histroy buffer should not be empty")
    }

    pub fn get_max(&self) -> f32 {
        self.history
            .peek()
            .iter()
            .cloned()
            .reduce(f32::max)
            .expect("Histroy buffer should not be empty")
    }

    pub fn get_average(&self) -> f32 {
        self.history.peek().iter().cloned().sum::<f32>() / self.history.peek().len() as f32
    }

    pub fn get_last(&self) -> f32 {
        *self.history.peek().last().unwrap()
    }
}
