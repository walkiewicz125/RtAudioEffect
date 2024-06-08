mod audio;
mod audio_analyzer;
mod logger;
mod ui;

use audio_analyzer::StreamAnalyzerReceiver;
use egui::Color32;
use egui_glfw::AppWindow;
use log::{error, info};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use service::Service;
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use ui::central_panel::HeatMapImage;

use crate::{
    audio::{audio_stream::AudioStream, AudioManager, AudioStreamConsumer},
    audio_analyzer::StreamAnalyzer,
    messages::Message,
    service::ServiceRegister,
    ui::ui_controller::UiController,
};

mod service;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
extern crate serializer;
pub mod messages;

fn main() {
    info!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let mut service_register = ServiceRegister::new();
    let service = service_register.add_service("RtAudioEffect");

    let mut context = AppContext::new(service);

    if context.run() {
        info!("RtAudioEffect exit successfully");
    } else {
        error!("RtAudioEffect exit with error");
    }
}

struct AppContext {
    audio_stream: Arc<Mutex<AudioStream>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
    app_window: AppWindow,
    ui_controller: UiController,
    service: Arc<Service>,
}

impl AppContext {
    fn new(service: Arc<Service>) -> AppContext {
        let audio_stream = Arc::new(Mutex::new(
            AudioStream::new(AudioManager::get_default_loopback().unwrap()).unwrap(),
        ));

        let analyzer = Arc::new(Mutex::new(StreamAnalyzer::new(
            Duration::from_secs_f32(0.002),
            Duration::from_secs_f32(1.0),
            4800,
            audio_stream.lock().unwrap().get_parameters(),
        )));

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
        ui_controller.set_text_styles(&app_window.egui_context, 15.0);

        AppContext {
            audio_stream,
            analyzer,
            app_window,
            ui_controller,
            service,
        }
    }

    fn run(&mut self) -> bool {
        let analyzer_clone = self.analyzer.clone();
        let analyzer_thread = thread::spawn(move || {
            while analyzer_clone.lock().unwrap().is_alive() {
                analyzer_clone.lock().unwrap().process_new_samples();
            }
        });

        // let mut connection = self.service.wait_for_client();

        // let msg: Message = connection.recv_message().into();
        // println!("Received: {:#?}", msg);

        self.audio_stream.lock().unwrap().start();

        let mut last_time = std::time::Instant::now();
        let mut elapsed_time = std::time::Duration::from_secs_f32(0.0);
        let mut filtered_elapsed_time = std::time::Duration::from_secs_f32(0.0);

        while !self.app_window.window.should_close() {
            elapsed_time = last_time.elapsed();
            last_time = std::time::Instant::now();

            filtered_elapsed_time =
                filtered_elapsed_time.mul_f32(0.98) + elapsed_time.mul_f32(0.02);
            let fps = 1.0 / filtered_elapsed_time.as_secs_f32();

            self.app_window.begin_frame();

            let egui_context = self.app_window.get_egui_context();

            self.ui_controller.update_data(elapsed_time);

            egui::CentralPanel::default().show(&egui_context, |ui| {
                ui.add(self.ui_controller.get_central_panel(fps));
            });

            self.app_window.end_frame();
        }

        self.audio_stream.lock().unwrap().stop();
        self.analyzer.lock().unwrap().kill();
        analyzer_thread.join().unwrap();

        true
    }
}
