use std::{
    ops::{Index, IndexMut},
    sync::Arc,
};

use log::trace;
use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::audio::ChannelSamples;

#[derive(Clone)]
pub struct Spectrum(Vec<f32>);

impl Spectrum {
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }
}

impl FromIterator<f32> for Spectrum {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        Spectrum(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a Spectrum {
    type Item = &'a f32;
    type IntoIter = std::slice::Iter<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a> IntoIterator for &'a mut Spectrum {
    type Item = &'a mut f32;
    type IntoIter = std::slice::IterMut<'a, f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
impl Index<usize> for Spectrum {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Spectrum {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Into<Vec<f32>> for Spectrum {
    fn into(self) -> Vec<f32> {
        self.0
    }
}

impl From<Vec<f32>> for Spectrum {
    fn from(data: Vec<f32>) -> Spectrum {
        Spectrum(data)
    }
}
impl From<&[f32]> for Spectrum {
    fn from(data: &[f32]) -> Spectrum {
        Spectrum(data.to_vec())
    }
}

struct Window {
    weights: Vec<f32>,       // Window coefficients
    sum_window: f32,         // Sum of window coefficients
    _sum_square_window: f32, // Sum of squares of window coefficients
    _nenbw: f32,             // Normalized Equivalent Noise Bandwidth
    _enbw: f32,              // Effective Noise Bandwidth
}

impl Window {
    fn new(width: usize, sampling_frequency: usize) -> Window {
        let window = Window::generate_hanning_window(width);
        let sum_window: f32 = window.iter().sum();
        let sum_square_window: f32 = window.iter().map(|v| v * v).sum();
        let nenbw = width as f32 * sum_square_window / (sum_window * sum_window);
        let enbw = nenbw * sampling_frequency as f32 / width as f32;
        Window {
            weights: window,
            sum_window,
            _sum_square_window: sum_square_window,
            _nenbw: nenbw,
            _enbw: enbw,
        }
    }

    fn generate_hanning_window(width: usize) -> Vec<f32> {
        apodize::nuttall_iter(width)
            .map(|v| v as f32)
            .collect::<Vec<f32>>()
    }
}

pub struct FftAnalyzer {
    spectrum_width: usize,
    window: Window,
    fft: Arc<dyn Fft<f32>>,
    work_buffer: Vec<Complex<f32>>,
}

impl FftAnalyzer {
    pub fn new(spectrum_width: usize, sampling_frequency: usize) -> FftAnalyzer {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(spectrum_width);
        let window = Window::new(spectrum_width, sampling_frequency);

        FftAnalyzer {
            spectrum_width,
            window,
            fft,
            work_buffer: vec![Complex { re: 0.0, im: 0.0 }; spectrum_width],
        }
    }

    pub fn analyze(&mut self, new_samples: &ChannelSamples) -> Spectrum {
        trace!(
            "Calculating FFT of new samples. Sample count {}",
            new_samples.inner().len()
        );

        for i in 0..self.spectrum_width {
            self.work_buffer[i].re = new_samples[i] * self.window.weights[i];
            self.work_buffer[i].im = 0.0;
        }

        self.fft.process(&mut self.work_buffer);

        let mut spectrum: Spectrum = self
            .work_buffer
            .iter()
            .map(|number| number.norm() / self.window.sum_window)
            .take(self.spectrum_width / 2)
            .collect();
        spectrum[0] = 0.0;

        spectrum
    }
}
