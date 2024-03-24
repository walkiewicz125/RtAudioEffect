use egui::{vec2, Align, Layout, Pos2, Rect, Ui, Vec2, WidgetText};

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

pub fn add_rows(ui: &mut Ui, num_of_rows: i32, add_contents: impl FnOnce(&mut [Ui])) {
    ui.scope(|ui| {
        let spacing = ui.spacing().item_spacing.y;
        let total_spacing = spacing * (num_of_rows as f32 - 1.0);
        let row_height = (ui.available_height() - total_spacing) / (num_of_rows as f32);
        let top_left = ui.max_rect().left_top();
        let top_right = ui.max_rect().right_top();

        let mut rows: Vec<Ui> = (0..num_of_rows)
            .map(|row_idx| {
                let pos = top_left + vec2(0.0, (row_idx as f32) * (row_height + spacing));
                let child_rect =
                    Rect::from_two_pos(pos, top_right + vec2(0.0, row_height + spacing));

                let mut row_ui = ui.child_ui(child_rect, Layout::left_to_right(Align::Center));
                row_ui.set_height(row_height);
                row_ui
            })
            .collect();

        add_contents(&mut rows);
    });
}
