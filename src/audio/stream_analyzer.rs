use std::sync::Arc;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use super::audio_buffer::AudioBuffer;

pub struct StreamAnalyzer {
    spectrum_width: usize,
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    window_weight: f32,
    work_buffer: Vec<Complex<f32>>,
    mean_spectrum: Vec<f32>,
}

impl StreamAnalyzer {
    pub fn new(spectrum_width: usize) -> StreamAnalyzer {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(spectrum_width);
        let window = StreamAnalyzer::generate_hanning_window(spectrum_width);
        let window_weight = window.iter().sum();
        StreamAnalyzer {
            spectrum_width,
            fft,
            window,
            window_weight,
            work_buffer: vec![Complex { re: 0.0, im: 0.0 }; spectrum_width],
            mean_spectrum: vec![0.0; spectrum_width / 2],
        }
    }
    pub fn analyze(&mut self, audio_buffer: &mut AudioBuffer) {
        let audio_data =
            audio_buffer.fetch_with_overlap(0, self.spectrum_width, self.spectrum_width / 2);

        if audio_data.is_empty() {
            return;
        }

        for i in 0..self.spectrum_width {
            self.work_buffer[i].re = audio_data[i] * self.window[i];
            self.work_buffer[i].im = 0.0;
        }

        self.fft.process(&mut self.work_buffer);

        let spectrum = self.get_spectrum();
        for i in 0..(self.spectrum_width / 2) {
            self.mean_spectrum[i] = self.mean_spectrum[i] * 0.9 + spectrum[i] * 0.1;
        }
    }

    pub fn get_spectrum(&self) -> Vec<f32> {
        self.work_buffer
            .iter()
            .map(|number| number.norm() / self.window_weight)
            .take(self.spectrum_width / 2)
            .collect()
    }
    pub fn get_mean_spectrum(&self) -> Vec<f32> {
        self.mean_spectrum.clone()
    }
}

// private implementation
impl StreamAnalyzer {
    fn generate_hanning_window(width: usize) -> Vec<f32> {
        apodize::hanning_iter(width)
            .map(|v| v as f32)
            .collect::<Vec<f32>>()
    }
}
