mod glfw_egui;
mod logger;
mod plot;
mod ui_controller;

mod audio;
mod audio_analyzer;
mod audio_processor;
use egui_glfw::AppWindow;
use log::info;
use ui_controller::Resolution;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::audio::{
    audio_stream::AudioStream, audio_stream_consumer::AudioStreamConsumer, AudioManager,
};
use crate::{audio_analyzer::StreamAnalyzer, ui_controller::UiController};

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

    let mut app_window = AppWindow::new_default(SCREEN_WIDTH, SCREEN_HEIGHT);

    while !app_window.window.should_close() {
        app_window.begin_frame();

        let egui_context = app_window.get_egui_context();
        egui::CentralPanel::default().show(&egui_context, |ui| {
            ui.heading("Hello World!");
        });
        app_window.end_frame();
    }
}

fn mainold() {
    info!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let audio_stream = Arc::new(Mutex::new(
        AudioStream::new(AudioManager::get_default_loopback().unwrap()).unwrap(),
    ));

    let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(
        Duration::from_secs_f32(0.01),
        Duration::from_secs_f32(1.0),
        4800,
        audio_stream.lock().unwrap().get_parameters(),
    )));

    audio_stream
        .lock()
        .unwrap()
        .add_stream_consumer(analyzer.clone());

    let mut ui_controller: UiController =
        UiController::new(analyzer.clone(), audio_stream.clone(), DEFAULT_RESOLUTION);

    audio_stream.lock().unwrap().start();

    let analyzer_clone = analyzer.clone();
    let analyzer_thread = thread::spawn(move || {
        while analyzer_clone.lock().unwrap().is_alive() {
            analyzer_clone.lock().unwrap().process_new_samples();
        }
    });

    while !ui_controller.is_closing() {
        ui_controller.render();
    }

    audio_stream.lock().unwrap().stop();
    analyzer.lock().unwrap().kill();
    analyzer_thread.join().unwrap();

    info!("Goodbye RtAudioEffect!");
}
