use std::sync::{Arc, Mutex};

use egui::{PaintCallback, Sense, Widget};
use egui_glfw::back_end::PaintCallbackFn;

use super::spectrogram_renderer::SpectrogramRenderer;

pub struct SpectrogramRendererWidget {
    pub renderer: Arc<Mutex<SpectrogramRenderer>>,
}

impl Widget for SpectrogramRendererWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.allocate_response(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;
        let callback = PaintCallback {
            rect,
            callback: std::sync::Arc::new(PaintCallbackFn::new(move |_info, _painter| {
                self.renderer
                    .lock()
                    .unwrap()
                    .set_render_size((rect.width() as u32, rect.height() as u32));
                self.renderer.lock().unwrap().render();
            })),
        };
        ui.painter().add(callback);

        response
    }
}
