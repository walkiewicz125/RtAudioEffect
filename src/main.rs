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

struct AppContext {
    audio_stream: Arc<Mutex<AudioStream>>,
    analyzer: Arc<Mutex<StreamAnalyzer>>,
    app_window: AppWindow,
    ui_controller: UiController,
}

extern "system" fn gl_debug_output(
    source: u32,
    gltype: u32,
    id: u32,
    severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut std::ffi::c_void,
) {
    unsafe {
        let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
        match severity {
            gl::DEBUG_SEVERITY_HIGH => {
                log::error!("OpenGL Error: {}", message);
            }
            gl::DEBUG_SEVERITY_MEDIUM => {
                log::warn!("OpenGL Warning: {}", message);
            }
            gl::DEBUG_SEVERITY_LOW => {
                log::info!("OpenGL Info: {}", message);
            }
            gl::DEBUG_SEVERITY_NOTIFICATION => {
                log::debug!("OpenGL Notification: {}", message);
            }
            _ => {
                log::trace!("OpenGL Message: {}", message);
            }
        }
    }
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

        audio_stream
            .lock()
            .unwrap()
            .add_stream_consumer(analyzer.clone());

        let app_window = AppWindow::new_default(SCREEN_WIDTH, SCREEN_HEIGHT);
        let ui_controller = UiController::new(analyzer.clone(), audio_stream.clone());
        ui_controller.set_text_styles(app_window.get_egui_context());

        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(gl_debug_output), std::ptr::null());
            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                std::ptr::null(),
                gl::TRUE,
            );
        }

        AppContext {
            audio_stream,
            analyzer,
            app_window,
            ui_controller,
        }
    }

    fn run(&mut self) {
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
    }
}

fn main() {
    info!("Hello RtAudioEffect!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Debug))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let context = AppContext::new().run();

    info!("Goodbye RtAudioEffect!");
}
