mod glfw_egui;
mod logger;
mod plot;
mod ui_controller;

mod audio;
mod audio_analyzer;
mod audio_processor;
use audio_processor::AudioProcessor;
use log::info;
use ui_controller::Resolution;

use std::{
    sync::{Arc, Mutex},
    thread,
};

use crate::ui_controller::UiController;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: Resolution = (SCREEN_WIDTH, SCREEN_HEIGHT);

fn main() {
    info!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let audio_processor = Arc::new(Mutex::new(AudioProcessor::new()));
    let mut ui_controller = UiController::new(audio_processor.clone(), DEFAULT_RESOLUTION);

    audio_processor.lock().unwrap().start();

    let thread = thread::spawn(|| {
        while true {
            audio_processor.lock().unwrap().update();
        }
    });

    // split stream control from stream processing
    // run stream processing in separate thread.
    // leave stream control here where gui is

    while !ui_controller.is_closing() {
        ui_controller.render();
    }
    audio_processor.lock().unwrap().stop();

    info!("Goodbye RtAudioEffect!");
}
