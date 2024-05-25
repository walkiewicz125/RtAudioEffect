use egui::{vec2, Align, Layout, Rect, Ui, WidgetText};

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

pub fn add_rows(ui: &mut Ui, num_of_rows: i32, add_contents: impl FnOnce(&mut [Ui])) {
    ui.scope(|ui| {
        let spacing = ui.spacing().item_spacing.y;
        let total_spacing = spacing * (num_of_rows as f32 - 1.0);
        let row_height = (ui.available_height() - total_spacing) / (num_of_rows as f32);
        let top_left = ui.max_rect().left_top();
        let top_right = ui.max_rect().right_top();

        let mut rows: Vec<Ui> = (0..num_of_rows)
            .map(|row_idx| {
                let child_top_left =
                    top_left + vec2(0.0, (row_idx as f32) * (row_height + spacing));
                let child_bottom_right =
                    top_right + vec2(0.0, (row_idx as f32 + 1.0) * (row_height + spacing));

                let child_rect = Rect::from_two_pos(child_top_left, child_bottom_right);

                let mut row_ui = ui.child_ui(child_rect, Layout::left_to_right(Align::Center));
                row_ui.set_height(row_height);
                row_ui
            })
            .collect();

        add_contents(&mut rows);
    });
}

pub fn add_columns(ui: &mut Ui, num_of_columns: i32, add_contents: impl FnOnce(&mut [Ui])) {
    ui.scope(|ui| {
        let spacing = ui.spacing().item_spacing.x;
        let total_spacing = spacing * (num_of_columns as f32 - 1.0);
        let column_width = (ui.available_width() - total_spacing) / (num_of_columns as f32);
        let top_left = ui.max_rect().left_top();
        let bottom_left = ui.max_rect().left_bottom();

        let mut columns: Vec<Ui> = (0..num_of_columns)
            .map(|column_idx| {
                let child_top_left =
                    top_left + vec2((column_idx as f32) * (column_width + spacing), 0.0);
                let child_bottom_right =
                    bottom_left + vec2((column_idx as f32 + 1.0) * (column_width + spacing), 0.0);
                let child_rect = Rect::from_two_pos(child_top_left, child_bottom_right);
                let mut column_ui = ui.child_ui(child_rect, Layout::top_down(Align::Center));
                column_ui.set_width(column_width);
                column_ui
            })
            .collect();

        add_contents(&mut columns);
    });
}
