use std::time::Duration;

pub struct AnalyzerParameters {
    pub spectrum_width: usize,
    pub refresh_time_in_samples: usize,
    pub number_of_spectrums_in_history: usize,
    pub refresh_time: Duration,
    pub spectrogram_duration: Duration,
}
