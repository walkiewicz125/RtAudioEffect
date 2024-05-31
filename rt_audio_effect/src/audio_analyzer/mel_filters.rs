use converters::{hz_to_mel, mel_to_hz};

struct MelFilter {
    weights: Vec<f32>,
}

impl MelFilter {
    fn new(weights: Vec<f32>) -> MelFilter {
        MelFilter {
            weights: Self::normalize(weights),
        }
    }

    fn normalize(weights: Vec<f32>) -> Vec<f32> {
        let sum: f32 = weights.iter().sum();
        weights.iter().map(|&weight| weight / sum).collect()
    }

    fn apply(&self, spectrum: &[f32]) -> f32 {
        self.weights
            .iter()
            .zip(spectrum)
            .map(|(&weight, &spectrum_value)| weight * spectrum_value)
            .sum()
    }
}

struct MelFilterBank {
    filters: Vec<MelFilter>,
}

impl MelFilterBank {
    const MIN_FREQUNCY: f32 = 0.0;
    const MAX_FREQUENCY: f32 = 20000.0;

    pub fn new(num_of_filters: usize, fft_width: usize, sample_rate: f32) -> MelFilterBank {
        let min_mel = hz_to_mel(Self::MIN_FREQUNCY);
        let max_mel = hz_to_mel(Self::MAX_FREQUENCY);

        // Create points of start/center/end points of each filter
        let mel_points = (0..=num_of_filters + 1)
            .map(|i| min_mel + i as f32 * (max_mel - min_mel) / (num_of_filters as f32))
            .collect::<Vec<f32>>();

        let hz_points = mel_points
            .iter()
            .map(|&mel| mel_to_hz(mel))
            .collect::<Vec<f32>>();
        let bin_points = hz_points
            .iter()
            .map(|&hz| (hz * fft_width as f32 / sample_rate as f32).floor() as usize)
            .collect::<Vec<usize>>();

        // Vec of filters. Filter = Vec of weights over spectrum
        let mut filters_weights = vec![vec![0.0; fft_width]; num_of_filters];

        for i in 1..=num_of_filters {
            let start = bin_points[i - 1];
            let center = bin_points[i];
            let end = bin_points[i + 1];

            for j in start..center {
                filters_weights[i - 1][j] = (j - start) as f32 / (center - start) as f32;
            }

            for j in center..end {
                filters_weights[i - 1][j] = (end - j) as f32 / (end - center) as f32;
            }
        }

        let filters = filters_weights
            .into_iter()
            .map(|weights| MelFilter::new(weights))
            .collect::<Vec<MelFilter>>();

        MelFilterBank { filters }
    }

    pub fn apply(&self, spectrum: &[f32]) -> Vec<f32> {
        self.filters
            .iter()
            .map(|filter| filter.apply(spectrum))
            .collect()
    }
}

mod converters {
    pub fn hz_to_mel(hz: f32) -> f32 {
        2595.0 * (1.0 + hz / 700.0).log10()
    }

    pub fn mel_to_hz(mel: f32) -> f32 {
        700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mel_filter_bank_new() {
        let num_of_filters = 10;
        let fft_width = 1024;
        let sample_rate = 44100.0;

        let mel_filter_bank = MelFilterBank::new(num_of_filters, fft_width, sample_rate);

        // Assert the number of filters
        assert_eq!(mel_filter_bank.filters.len(), num_of_filters);

        // Assert the weights of each filter
        for filter in mel_filter_bank.filters {
            assert_eq!(filter.weights.len(), fft_width);
            filter.weights.iter().for_each(|&weight| {
                assert!(weight >= 0.0 && weight <= 1.0);
            });
        }
    }

    #[test]
    fn test_mel_filter_normalize() {
        let weights = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0];

        let normalized_weights = MelFilter::normalize(weights);

        let expected_normalized_weights =
            vec![0.0, 0.0625, 0.125, 0.1875, 0.25, 0.1875, 0.125, 0.0625, 0.0];

        assert_eq!(normalized_weights, expected_normalized_weights);
    }

    #[test]
    fn test_mel_filter_apply() {
        let weights = vec![0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25, 0.0];
        let mel_filter = MelFilter::new(weights);

        let spectrum = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
        let filtered_spectrum = mel_filter.apply(&spectrum);

        let expected_normalized_weights =
            vec![0.0, 0.0625, 0.125, 0.1875, 0.25, 0.1875, 0.125, 0.0625, 0.0];

        let expected_value = expected_normalized_weights
            .iter()
            .zip(spectrum.iter())
            .map(|(&weight, &spectrum_value)| weight * spectrum_value)
            .sum();

        assert_eq!(filtered_spectrum, expected_value);
    }

    // Validated with https://www.homepages.ucl.ac.uk/~sslyjjt/speech/Mel2Hz.html
    #[test]
    fn test_hz_to_mel() {
        assert_eq!(converters::hz_to_mel(20.0), 31.748478);
        assert_eq!(converters::hz_to_mel(400.0), 509.38458);
        assert_eq!(converters::hz_to_mel(1000.0), 999.98553);
        assert_eq!(converters::hz_to_mel(2450.0), 1695.0864);
        assert_eq!(converters::hz_to_mel(14000.0), 3431.159);
        assert_eq!(converters::hz_to_mel(20000.0), 3816.9136);
    }

    #[test]
    fn test_mel_to_hz() {
        assert_eq!(converters::mel_to_hz(60.0), 38.27716);
        assert_eq!(converters::mel_to_hz(500.0), 390.87817);
        assert_eq!(converters::mel_to_hz(1000.0), 1000.0219);
        assert_eq!(converters::mel_to_hz(2000.0), 3428.6772);
        assert_eq!(converters::mel_to_hz(3800.0), 19691.658);
    }
}
