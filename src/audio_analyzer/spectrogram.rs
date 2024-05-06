use std::sync::Arc;

use crate::audio::StreamParameters;

use super::{AnalyzerParameters, ManyChannelsSpectrums, Spectrum};

pub type Magnitude = f32;

#[derive(Clone)]
pub struct TimeSeries<T> {
    data: Vec<T>,
    length: usize,
    width: usize,
    total_size: usize,
}

impl<T> TimeSeries<T>
where
    T: Clone,
{
    pub fn new(length: usize, width: usize, default_value: T) -> TimeSeries<T> {
        TimeSeries {
            data: vec![default_value; length * width],
            length: length,
            width: width,
            total_size: length * width,
        }
    }

    pub fn push(&mut self, data: Vec<T>) {
        self.data.extend(data);

        if self.data.len() >= self.length * self.width {
            self.data.drain(0..self.data.len() - self.total_size);
        }
    }

    pub fn get_total_len(&self) -> usize {
        self.total_size
    }

    pub fn get_length(&self) -> usize {
        self.length
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_last(&self) -> &[T] {
        &self.data[self.data.len() - self.width..]
    }
    pub fn get_data(&self) -> &[T] {
        self.data.as_slice()
    }
}

pub struct MultiChannel<T> {
    pub channels: Vec<T>,
}

impl<T> MultiChannel<T>
where
    T: Clone,
{
    pub fn new(channel_count: usize, default_value: T) -> MultiChannel<T> {
        MultiChannel {
            channels: vec![default_value; channel_count],
        }
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }
}

pub struct Spectrogram {
    stream_parameters: Arc<StreamParameters>,
    analyzer_parameters: Arc<AnalyzerParameters>,
    spectrum_history: MultiChannel<TimeSeries<Magnitude>>,
}

impl Spectrogram {
    pub fn new(
        analyzer_parameters: Arc<AnalyzerParameters>,
        stream_parameters: Arc<StreamParameters>,
    ) -> Spectrogram {
        Spectrogram {
            stream_parameters: stream_parameters.clone(),
            analyzer_parameters: analyzer_parameters.clone(),
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

    pub fn push_spectrums(&mut self, spectrums: ManyChannelsSpectrums) {
        assert!(spectrums.len() == self.stream_parameters.channels as usize);
        assert!(spectrums.len() == self.spectrum_history.len());

        for (i, data) in spectrums.into_iter().enumerate() {
            self.spectrum_history.channels[i].push(data);
        }
    }

    pub fn get_latest_spectrum(&self) -> ManyChannelsSpectrums {
        let mut spectrums = vec![];
        for channel in &self.spectrum_history.channels {
            spectrums.push(channel.get_last().to_vec());
        }
        spectrums
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
