use egui::{vec2, Align, Layout, Rect, Ui};

pub fn add_rows<R>(ui: &mut Ui, num_of_rows: i32, add_contents: impl FnOnce(&mut [Ui]) -> R) -> R {
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

        add_contents(&mut rows)
    })
    .inner
}

pub fn add_columns<R>(
    ui: &mut Ui,
    num_of_columns: i32,
    add_contents: impl FnOnce(&mut [Ui]) -> R,
) -> R {
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

        add_contents(&mut columns)
    })
    .inner
}
