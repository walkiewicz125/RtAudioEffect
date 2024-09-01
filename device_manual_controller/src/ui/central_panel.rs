use std::sync::{Arc, Mutex};

use egui::{Response, Ui, Vec2, Widget};
use egui_addons::joystick::Joystick;

use super::device_states::DeviceStates;

pub struct CentralPanel {
    fps: f32,
    states: Arc<Mutex<DeviceStates>>,
}

impl CentralPanel {
    pub fn build(fps: f32, states: Arc<Mutex<DeviceStates>>) -> Self {
        Self { fps, states }
    }
}

impl Widget for CentralPanel {
    fn ui(mut self, ui: &mut Ui) -> Response {
        // let mut states = (*self.states.lock().unwrap()).clone();
        // println!("Available rect: {:?}", ui.available_size());
        ui.vertical_centered(|ui| {
            ui.group(|ui| {
                let response = ui.label("joystick");
                response.union(ui.add(Joystick::new(
                    &mut self.states.lock().unwrap().joystick1,
                    Vec2::new(200.0, 200.0),
                )));
                if ui.button("center").clicked() {
                    self.states.lock().unwrap().joystick1 = Vec2::ZERO;
                }
                response
            })
            .inner
        })
        .inner
    }
}
