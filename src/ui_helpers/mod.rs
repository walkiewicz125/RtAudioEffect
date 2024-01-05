use egui::WidgetText;

pub mod ui_helpers {
    use egui::WidgetText;

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
}
