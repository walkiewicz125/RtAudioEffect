pub mod ui_helpers {
    pub fn text_input_f32(
        ui: &mut egui::Ui,
        label_id: &egui::Id,
        value_text: &mut String,
    ) -> Option<f32> {
        let old_value = value_text.clone();
        let response = ui.text_edit_singleline(value_text).labelled_by(*label_id);

        if response.changed() {
            if let Err(_) = value_text.parse::<f32>() {
                *value_text = old_value;
            }
        }
        if response.lost_focus() {
            if let Ok(value) = value_text.parse::<f32>() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    return Some(value);
                }
            }
        }

        None
    }
    pub fn text_input_u32(
        ui: &mut egui::Ui,
        label_id: &egui::Id,
        value_text: &mut String,
    ) -> Option<u32> {
        let old_value = value_text.clone();
        let response = ui.text_edit_singleline(value_text).labelled_by(*label_id);

        if response.changed() {
            if let Err(_) = value_text.parse::<f32>() {
                *value_text = old_value;
            }
        }
        if response.lost_focus() {
            if let Ok(value) = value_text.parse::<u32>() {
                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    return Some(value);
                }
            }
        }

        None
    }
}
