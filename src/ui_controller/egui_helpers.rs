use egui::{FontId, Pos2, Rect, TextStyle, Vec2, WidgetText};

use crate::glfw_egui::{egui_glfw, glfw_painter};

pub fn number_input<T>(
    ui: &mut egui::Ui,
    label: impl Into<WidgetText>,
    value_text: &mut String,
) -> Option<T>
where
    T: std::str::FromStr,
{
    let label = ui.label(label);
    let old_value = value_text.clone();
    let response = ui.text_edit_singleline(value_text).labelled_by(label.id);

    if response.changed() {
        if let Err(_) = value_text.parse::<T>() {
            *value_text = old_value;
        }
    }
    if response.lost_focus() {
        if let Ok(value) = value_text.parse::<T>() {
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                return Some(value);
            }
        }
    }

    None
}

pub fn initialize_egui(
    window: &mut glfw::Window,
) -> (
    glfw_painter::Painter,
    egui::Context,
    egui_glfw::EguiInputState,
) {
    let painter = glfw_painter::Painter::new(window);
    let egui_ctx = egui::Context::default();

    let (width, height) = window.get_framebuffer_size();
    let native_pixels_per_point = window.get_content_scale().0;

    let egui_input_state = egui_glfw::EguiInputState::new(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::new(0f32, 0f32),
            Vec2 {
                x: width as f32,
                y: height as f32,
            } / native_pixels_per_point,
        )),
        ..Default::default() // todo: add pixels_per_point
    });

    (painter, egui_ctx, egui_input_state)
}
