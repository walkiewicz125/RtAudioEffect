use std::sync::Arc;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use crate::audio::OneChannelSamples;

pub type Spectrum = Vec<f32>;
pub type ManyChannelsSpectrums = Vec<Spectrum>;
type WindowWeitghts = Vec<f32>;

pub struct SpectrumAnalyzer {
    spectrum_width: usize,
    window: Vec<f32>,
    window_weight: f32,
    fft: Arc<dyn Fft<f32>>,
    work_buffer: Vec<Complex<f32>>,
}

impl SpectrumAnalyzer {
    pub fn new(spectrum_width: usize) -> SpectrumAnalyzer {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(spectrum_width);
        let window = SpectrumAnalyzer::generate_hanning_window(spectrum_width);
        let window_weight = window.iter().sum();

        SpectrumAnalyzer {
            spectrum_width,
            window,
            window_weight,
            fft,
            work_buffer: vec![Complex { re: 0.0, im: 0.0 }; spectrum_width],
        }
    }

    fn generate_hanning_window(width: usize) -> WindowWeitghts {
        apodize::hanning_iter(width)
            .map(|v| v as f32)
            .collect::<WindowWeitghts>()
    }

    pub fn analyze(&mut self, new_samples: &OneChannelSamples) -> Spectrum {
        for i in 0..self.spectrum_width {
            self.work_buffer[i].re = new_samples[i] * self.window[i];
            self.work_buffer[i].im = 0.0;
        }

        self.fft.process(&mut self.work_buffer);

        self.work_buffer
            .iter()
            .map(|number| number.norm() / self.window_weight)
            .take(self.spectrum_width / 2)
            .collect()
    }
}
