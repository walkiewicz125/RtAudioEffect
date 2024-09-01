mod logger;
mod ui;

use egui_glfw::AppWindow;
use log::{error, info};
use service::AudioHeadlightService;
use std::sync::{Arc, Mutex};

use crate::{service::ServiceRegister, ui::ui_controller::UiController};

mod service;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
extern crate serializer;

fn main() {
    info!("Hello DeviceManualcontroller!");

    if let Err(err) =
        log::set_logger(&logger::LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
    {
        eprintln!("log::set_logger failed: {err:#?}");
    }

    let mut service_register = ServiceRegister::new();
    let service = service_register.add_service("RtAudioEffect");

    let mut context = AppContext::new(service);

    if context.run() {
        info!("DeviceManualcontroller exit successfully");
    } else {
        error!("DeviceManualcontroller exit with error");
    }
}

struct AppContext {
    app_window: AppWindow,
    ui_controller: UiController,
    service: Arc<Mutex<AudioHeadlightService>>,
}

impl AppContext {
    fn new(service: Arc<Mutex<AudioHeadlightService>>) -> AppContext {
        let app_window = AppWindow::new_default(SCREEN_WIDTH, SCREEN_HEIGHT);

        let ui_controller = UiController::new();
        ui_controller.set_text_styles(&app_window.egui_context, 15.0);

        AppContext {
            app_window,
            ui_controller,
            service,
        }
    }

    fn run(&mut self) -> bool {
        let mut last_time = std::time::Instant::now();
        let mut filtered_elapsed_time = std::time::Duration::from_secs_f32(0.0);

        self.service.lock().unwrap().start();
        while !self.app_window.window.should_close() {
            let elapsed_time = last_time.elapsed();
            last_time = std::time::Instant::now();

            filtered_elapsed_time =
                filtered_elapsed_time.mul_f32(0.98) + elapsed_time.mul_f32(0.02);
            let fps = 1.0 / filtered_elapsed_time.as_secs_f32();

            self.app_window.begin_frame();

            let egui_context = self.app_window.get_egui_context();

            self.ui_controller.update_data(elapsed_time);

            egui::CentralPanel::default().show(&egui_context, |ui| {
                ui.add(self.ui_controller.get_central_panel(fps))
            });

            self.app_window.end_frame();
            self.service.lock().unwrap().set_servo(
                service::ServoId::Servo1,
                self.ui_controller.states.lock().unwrap().joystick1.x,
            );

            self.service.lock().unwrap().set_servo(
                service::ServoId::Servo2,
                self.ui_controller.states.lock().unwrap().joystick1.y,
            );

            // add sleep to reduce cpu usage
            std::thread::sleep(std::time::Duration::from_secs_f32(0.001));
        }

        true
    }
}
