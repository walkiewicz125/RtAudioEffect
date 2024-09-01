use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use egui::{FontData, FontDefinitions, FontFamily, FontId, TextStyle, Vec2};

use super::{central_panel::CentralPanel, device_states::DeviceStates};

pub struct UiController {
    pub states: Arc<Mutex<DeviceStates>>,
}

impl UiController {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(DeviceStates {
                joystick1: Vec2::new(0.0, 0.0),
            })),
        }
    }

    pub fn update_data(&self, time_step: Duration) {}

    pub fn get_central_panel(&self, fps: f32) -> CentralPanel {
        CentralPanel::build(fps, self.states.clone())
    }

    pub fn set_text_styles(&self, egui_context: &egui::Context, font_size: f32) {
        let mut fonts = FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters):
        fonts.font_data.insert(
            "DejaVuSans".to_owned(),
            FontData::from_static(include_bytes!("fonts/DejaVuSans.ttf")),
        ); // .ttf and .otf supported

        // Put my font first (highest priority):
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "DejaVuSans".to_owned());

        egui_context.set_fonts(fonts);

        egui_context.style_mut(|x| {
            x.text_styles.insert(
                TextStyle::Body,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Button,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Heading,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Small,
                FontId::new(font_size * 0.8, egui::FontFamily::Proportional),
            );
        });
    }
}
