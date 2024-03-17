mod glfw_egui;
mod logger;
mod plot;

mod audio;
mod audio_analyzer;
mod audio_processor;
use audio_processor::AudioProcessor;

pub mod ui_helpers;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: (u32, u32) = (SCREEN_WIDTH, SCREEN_HEIGHT);

fn main() {
    println!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let mut audio_processor = AudioProcessor::new();

    audio_processor.start();

    let start_time = Instant::now();

    // for tests
    while (Instant::now() - start_time) < Duration::from_secs_f32(3.0) {
        audio_processor.update();
    }

    audio_processor.stop();

    println!("Goodbye RtAudioEffect!");
}
