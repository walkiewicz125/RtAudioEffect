mod glfw_egui;
mod logger;
mod plot;
mod ui_controller;

mod audio;
mod audio_analyzer;
mod audio_processor;
use audio_processor::AudioProcessor;
use ui_controller::Resolution;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::ui_controller::UiController;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: Resolution = (SCREEN_WIDTH, SCREEN_HEIGHT);

fn main() {
    println!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let audio_processor = Arc::new(Mutex::new(AudioProcessor::new()));
    let mut ui_controller = UiController::new(audio_processor.clone(), DEFAULT_RESOLUTION);

    audio_processor.lock().unwrap().start();

    let start_time = Instant::now();
    // for tests
    while (Instant::now() - start_time) < Duration::from_secs_f32(3.0) {
        audio_processor.lock().unwrap().update();
        if ui_controller.is_closing() {
            break;
        } else {
            ui_controller.update();
        }
    }
    audio_processor.lock().unwrap().stop();

    println!("Goodbye RtAudioEffect!");
}
