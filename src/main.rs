mod glfw_egui;
mod logger;
mod plot;
mod ui_controller;

mod audio;
mod audio_analyzer;
mod audio_processor;
use log::{debug, info};
use ui_controller::Resolution;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{audio_processor::AudioStream, ui_controller::UiController};

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

    let audio_processor = Arc::new(Mutex::new(AudioStream::new()));
    let mut ui_controller: UiController =
        UiController::new(audio_processor.clone(), DEFAULT_RESOLUTION);

    audio_processor.lock().unwrap().start();
    let receiver: Arc<Mutex<audio::AudioStreamReceiver>> =
        audio_processor.lock().unwrap().stream_receiver.clone();
    let _thread = thread::spawn(move || loop {
        receiver.lock().unwrap().update();
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
