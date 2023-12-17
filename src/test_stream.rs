use std::{f32::consts::PI, time::Instant};

use crate::traits::stream::{NormalizedMixedChanelData, Stream, StreamData};
enum StreamState {
    STOPPED,
    STARTED,
}
pub struct TestStream {
    state: StreamState,
    frequency: f32,
    start_time: Instant,
    last_fetch_time: Instant,
}

impl TestStream {
    const SAMPLING_FREQUENCY: f32 = 44100.0;
    const CHANEL_COUNT: i8 = 2;

    pub fn new() -> TestStream {
        TestStream {
            state: StreamState::STOPPED,
            frequency: 0.0,
            start_time: Instant::now(),
            last_fetch_time: Instant::now(),
        }
    }
    pub fn start(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.start_time = Instant::now();
        self.last_fetch_time = Instant::now();
        self.state = StreamState::STARTED;
    }

    pub fn stop(&mut self) {
        self.state = StreamState::STOPPED;
    }
}

impl Stream for TestStream {
    fn fetch_data(&mut self) -> StreamData {
        let start_phase_time = (self.last_fetch_time - self.start_time).as_secs_f32();
        self.last_fetch_time = Instant::now();

        let elapsed = self.last_fetch_time.elapsed().as_secs_f32();
        let samples_to_generate = (elapsed * TestStream::SAMPLING_FREQUENCY) as isize;

        let phase_step = (self.frequency * 2.0 * PI) / TestStream::SAMPLING_FREQUENCY;
        let mut phase =
            (start_phase_time * self.frequency * 2.0 * PI) / TestStream::SAMPLING_FREQUENCY;
        let mut samples: Vec<f32> = vec![];
        for _i in 0..samples_to_generate {
            samples.push((phase).sin());
            samples.push((phase).cos());
            phase += phase_step;
        }

        StreamData::NormalizedMixedChanel(NormalizedMixedChanelData {
            stream_data: samples,
            num_of_chanels: TestStream::CHANEL_COUNT,
            sampling_frequency: TestStream::SAMPLING_FREQUENCY,
        })
    }
}

#[cfg(test)]
mod tests {
    use core::time;
    use std::{f32::consts::PI, thread};

    use crate::traits::stream::{Stream, StreamData};

    use super::TestStream;

    #[test]
    fn test_update_start_time_and_frequency() {
        let mut stream = TestStream::new();
        let freq = 1000.0;

        stream.start(freq);
        let time_1 = stream.last_fetch_time.clone();

        stream.start(freq);
        let time_2 = stream.last_fetch_time.clone();

        assert_ne!(time_1, time_2);
        assert!(time_2 > time_1);
        assert_eq!(freq, stream.frequency);
    }

    #[test]
    fn test_fetch_data() {
        let mut stream = TestStream::new();
        let freq = 1000.0;

        stream.start(freq);

        thread::sleep(time::Duration::from_secs_f32(1.0));

        let StreamData::NormalizedMixedChanel(data) = stream.fetch_data();
        assert_eq!(data.num_of_chanels, 2);
        assert_eq!(data.sampling_frequency, 44100.0);

        let mut left_chanel: Vec<f32> = Vec::with_capacity(data.stream_data.len());
        let mut right_chanel: Vec<f32> = Vec::with_capacity(data.stream_data.len());

        for (sample, value) in data.stream_data.iter().enumerate() {
            if sample % 2 == 0 {
                left_chanel.push(value.clone());
            } else {
                right_chanel.push(value.clone());
            }
        }

        let phase_step = (freq * 2.0 * PI) / TestStream::SAMPLING_FREQUENCY;
        let mut phase: f32 = 0.0;

        for i in 0..left_chanel.len() {
            assert_eq!(left_chanel[i], phase.sin());
            assert_eq!(right_chanel[i], phase.cos());
            phase += phase_step;
        }

        thread::sleep(time::Duration::from_secs_f32(1.0));

        let StreamData::NormalizedMixedChanel(data) = stream.fetch_data();
        assert_eq!(data.num_of_chanels, 2);
        assert_eq!(data.sampling_frequency, 44100.0);

        let mut left_chanel: Vec<f32> = Vec::with_capacity(data.stream_data.len());
        let mut right_chanel: Vec<f32> = Vec::with_capacity(data.stream_data.len());

        for (sample, value) in data.stream_data.iter().enumerate() {
            if sample % 2 == 0 {
                left_chanel.push(value.clone());
            } else {
                right_chanel.push(value.clone());
            }
        }

        let phase_step = (freq * 2.0 * PI) / TestStream::SAMPLING_FREQUENCY;
        let mut phase = (stream.last_fetch_time - stream.start_time).as_secs_f32();

        for i in 0..left_chanel.len() {
            assert_eq!(left_chanel[i], phase.sin());
            assert_eq!(right_chanel[i], phase.cos());
            phase += phase_step;
        }
    }
}
