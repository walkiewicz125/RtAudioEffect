mod audio;
mod audio_analyzer;
mod audio_annotator;
mod logger;
mod ui;

use audio_analyzer::{ManyChannelsSpectrums, StreamAnalyzerReceiver};
use audio_annotator::StreamAnalyzerAnnotator;
use egui::Color32;
use egui_glfw::AppWindow;
use log::{error, info};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use ui::central_panel::HeatMapImage;

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

    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    let rt_audio_efect_service = ServiceInfo::new(
        "_RtAudioEffect._udp.local.",
        "RtAudioEffect",
        "RtAudioEffect.local.",
        "127.0.0.1",
        12337,
        None,
    )
    .unwrap()
    .enable_addr_auto();

    mdns.register(rt_audio_efect_service)
        .expect("Failed to register RtAudioEffect service in mDNS deamon");

    let mut context = AppContext::new();

    if context.run() {
        info!("RtAudioEffect exit successfully");
    } else {
        error!("RtAudioEffect exit with error");
    }
}

struct AppContext {
    audio_stream: Arc<Mutex<AudioStream>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
    annotator: Arc<Mutex<StreamAnalyzerAnnotator>>,
    app_window: AppWindow,
    ui_controller: UiController,
}

impl AppContext {
    fn new() -> AppContext {
        let audio_stream = Arc::new(Mutex::new(
            AudioStream::new(AudioManager::get_default_loopback().unwrap()).unwrap(),
        ));

        let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(
            Duration::from_secs_f32(0.01),
            Duration::from_secs_f32(1.0),
            4800,
            audio_stream.lock().unwrap().get_parameters(),
        )));
        let annotator = Arc::new(Mutex::new(StreamAnalyzerAnnotator::new(
            analyzer.lock().unwrap().get_analyzer_parameters(),
            Duration::from_secs_f32(1.0),
            audio_stream.lock().unwrap().get_parameters().channels as usize,
        )));

        analyzer
            .lock()
            .unwrap()
            .register_receiver(annotator.clone());

        audio_stream
            .lock()
            .unwrap()
            .add_stream_consumer(analyzer.clone());

        let app_window = AppWindow::new_default(SCREEN_WIDTH, SCREEN_HEIGHT);

        let ui_controller = UiController::new(
            analyzer.clone(),
            audio_stream.clone(),
            HeatMapImage::new(app_window.get_egui_context()),
        );
        ui_controller.set_text_styles(&app_window.egui_context, 20.0);

        AppContext {
            audio_stream,
            analyzer,
            annotator,
            app_window,
            ui_controller,
        }
    }

    fn run(&mut self) -> bool {
        let analyzer_clone = self.analyzer.clone();
        let analyzer_thread = thread::spawn(move || {
            while analyzer_clone.lock().unwrap().is_alive() {
                analyzer_clone.lock().unwrap().process_new_samples();
            }
        });

        self.audio_stream.lock().unwrap().start();

        while !self.app_window.window.should_close() {
            self.app_window.begin_frame();

            let egui_context = self.app_window.get_egui_context();

            self.ui_controller.update_data();

            egui::CentralPanel::default().show(&egui_context, |ui| {
                ui.add(self.ui_controller.get_central_panel());
            });

            self.app_window.end_frame();
        }

        self.audio_stream.lock().unwrap().stop();
        self.analyzer.lock().unwrap().kill();
        analyzer_thread.join().unwrap();

        true
    }
}
