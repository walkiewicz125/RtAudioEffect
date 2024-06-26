use std::time::Duration;

pub struct AnalyzerParameters {
    pub spectrum_width: usize,
    pub refresh_time_in_samples: usize,
    pub length_of_history: usize,
    pub refresh_time: Duration,
    pub spectrogram_duration: Duration,
    pub sample_rate: u32,
}
