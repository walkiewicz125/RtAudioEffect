mod glfw_egui;
mod plot;

mod app;
pub mod ui_helpers;
use app::RtAudioEffect;
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
    println!("Hello, world!");
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug));
    let mut app_context = RtAudioEffect::new();
    app_context.run();

    println!("Goodbye, world!");
}
