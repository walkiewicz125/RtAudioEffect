mod audio;
mod audio_analyzer;
mod logger;
mod ui;

use egui_glfw::AppWindow;
use log::info;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    audio::{audio_stream::AudioStream, AudioManager, AudioStreamConsumer},
    audio_analyzer::StreamAnalyzer,
    ui::ui_controller::UiController,
};

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;

fn main() {
    info!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let mut app_window = AppWindow::new_default(SCREEN_WIDTH, SCREEN_HEIGHT);

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

    audio_stream.lock().unwrap().start();
    let ui_controller = UiController::new(analyzer.clone(), audio_stream);
    ui_controller.set_text_styles(app_window.get_egui_context());

    let analyzer_clone = analyzer.clone();
    let analyzer_thread = thread::spawn(move || {
        while analyzer_clone.lock().unwrap().is_alive() {
            analyzer_clone.lock().unwrap().process_new_samples();
        }
    });

    while !app_window.window.should_close() {
        app_window.begin_frame();
        let egui_context = app_window.get_egui_context();
        ui_controller.update_data();
        egui::CentralPanel::default().show(&egui_context, |ui| {
            ui.add(ui_controller.get_central_panel());
        });

        app_window.end_frame();
    }
}
