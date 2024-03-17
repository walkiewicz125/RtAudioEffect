mod glfw_egui;
mod plot;

mod audio_analyzer;
mod audio_processor;
pub mod ui_helpers;
use std::time::{Duration, Instant};

use audio_processor::AudioProcessor;
mod audio;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: (u32, u32) = (SCREEN_WIDTH, SCREEN_HEIGHT);

use log::{Level, Metadata, Record};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "{} | {} | {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    println!("Hello RtAudioEffect!");

    if let Err(err) = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
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
