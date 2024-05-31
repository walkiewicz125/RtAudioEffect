use std::sync::Arc;

use log::trace;
use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::audio::OneChannelSamples;

use super::{Magnitude, MultiChannel};

pub type Spectrum = Vec<f32>;
pub type ManyChannelsSpectrums = Vec<Spectrum>;
type WindowWeitghts = Vec<f32>;

struct Window {
    window: Vec<f32>,       // Window coefficients
    sum_window: f32,        // Sum of window coefficients
    sum_square_window: f32, // Sum of squares of window coefficients
    nenbw: f32,             // Normalized Equivalent Noise Bandwidth
    enbw: f32,              // Effective Noise Bandwidth
}

impl Window {
    fn new(width: usize, sampling_frequency: usize) -> Window {
        let window = Window::generate_hanning_window(width);
        let sum_window: f32 = window.iter().sum();
        let sum_square_window: f32 = window.iter().map(|v| v * v).sum();
        let nenbw = width as f32 * sum_square_window / (sum_window * sum_window);
        let enbw = nenbw * sampling_frequency as f32 / width as f32;
        Window {
            window,
            sum_window,
            sum_square_window,
            nenbw,
            enbw,
        }
    }

    fn generate_hanning_window(width: usize) -> Vec<f32> {
        apodize::nuttall_iter(width)
            .map(|v| v as f32)
            .collect::<Vec<f32>>()
    }
}

pub struct SpectrumAnalyzer {
    spectrum_width: usize,
    window: Window,
    fft: Arc<dyn Fft<f32>>,
    work_buffer: Vec<Complex<f32>>,
}

impl SpectrumAnalyzer {
    pub fn new(spectrum_width: usize, sampling_frequency: usize) -> SpectrumAnalyzer {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(spectrum_width);
        let window = Window::new(spectrum_width, sampling_frequency);

        SpectrumAnalyzer {
            spectrum_width,
            window,
            fft,
            work_buffer: vec![Complex { re: 0.0, im: 0.0 }; spectrum_width],
        }
    }

    pub fn analyze(&mut self, new_samples: &OneChannelSamples) -> Spectrum {
        trace!(
            "Calculating FFT of new samples. Sample count {}",
            new_samples.len()
        );

        for i in 0..self.spectrum_width {
            self.work_buffer[i].re = new_samples[i] * self.window.window[i];
            self.work_buffer[i].im = 0.0;
        }

        self.fft.process(&mut self.work_buffer);

        let mut spectrum: Spectrum = self
            .work_buffer
            .iter()
            .map(|number| number.norm() / self.window.sum_window)
            .take(self.spectrum_width / 2)
            .collect();
        spectrum[0] = 0.0; // TODO: keep?

        spectrum
    }
}
