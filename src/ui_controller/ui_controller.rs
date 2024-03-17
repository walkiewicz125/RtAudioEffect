use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Instant,
};

use egui::{FontId, TextStyle};
use glfw::{Context, Glfw, WindowEvent};

use crate::glfw_egui::{egui_glfw, glfw_painter};

use super::{egui_helpers, helpers};

pub type Resolution = (u32, u32);

struct UiWindow {
    glfw_context: Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    painter: glfw_painter::Painter,
    egui_context: egui::Context,
    egui_input_state: egui_glfw::EguiInputState,
    start_time: Instant,
}

impl UiWindow {
    fn new(resolution: Resolution) -> UiWindow {
        let mut glfw_context = helpers::create_glfx_context();
        let (mut window, events) = helpers::create_window(&mut glfw_context, resolution);
        let (painter, egui_context, egui_input_state) = egui_helpers::initialize_egui(&mut window);
        Self::apply_ui_style(&egui_context);

        UiWindow {
            glfw_context,
            window,
            events,
            painter,
            egui_context,
            egui_input_state,
            start_time: Instant::now(),
        }
    }

    fn apply_ui_style(egui_context: &egui::Context) {
        egui_context.style_mut(|x| {
            x.text_styles.insert(
                TextStyle::Body,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Button,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Heading,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
        });
    }

    fn should_close(&self) -> bool {
        self.window.should_close()
    }

    fn update(&mut self) {
        if !self.window.should_close() {
            self.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
            self.egui_context
                .begin_frame(self.egui_input_state.input.take());

            self.draw_top_panel();

            self.finalize_frame();

            self.window.swap_buffers();
            self.glfw_context.poll_events();
        }
    }

    fn draw_top_panel(&mut self) {
        egui::TopBottomPanel::top("Top").show(&self.egui_context, |ui| {
            ui.menu_button("File", |ui| {
                let _ = ui.button("test 1");
                ui.separator();
                let _ = ui.button("test 1");
            })
        });
    }

    fn finalize_frame(&mut self) {
        let native_pixels_per_point = self.window.get_content_scale().0;
        self.egui_context
            .set_pixels_per_point(native_pixels_per_point);
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point: native_pixels_per_point,
            viewport_output: _,
        } = self.egui_context.end_frame();

        //Handle cut, copy text from egui
        if !platform_output.copied_text.is_empty() {
            egui_glfw::copy_to_clipboard(&mut self.egui_input_state, platform_output.copied_text);
        }

        //Note: passing a bg_color to paint_jobs will clear any previously drawn stuff.
        //Use this only if egui is being used for all drawing and you aren't mixing your own Open GL
        //drawing calls with it.
        let clipped_shapes = self
            .egui_context
            .tessellate(shapes, native_pixels_per_point);
        self.painter
            .paint_and_update_textures(1.0, &clipped_shapes, &textures_delta);

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Close => self.window.set_should_close(true),
                glfw::WindowEvent::Size(width, height) => {
                    self.painter.set_size(width as _, height as _);
                    egui_glfw::handle_event(event, &mut self.egui_input_state);
                }
                _ => {
                    // println!("{:?}", event);
                    egui_glfw::handle_event(event, &mut self.egui_input_state);
                }
            }
        }
    }
}

pub struct UiController {
    window: UiWindow,
    audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
}
pub trait AudioAnalyzysProvider {}

impl UiController {
    pub fn new(
        audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        resolution: Resolution,
    ) -> UiController {
        let window = UiWindow::new(resolution);

        UiController {
            window,
            audio_analyzis_provider,
        }
    }

    pub fn update(&mut self) {
        self.window.update()
    }

    pub fn is_closing(&self) -> bool {
        self.window.should_close()
    }
}
