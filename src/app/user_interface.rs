use super::RtAudioEffect;
use crate::{
    glfw_egui::{egui_glfw, glfw_painter},
    ui_helpers::ui_helpers::number_input,
};
use egui::{self, FontId, Pos2, Rect, TextStyle, Vec2};
use glfw;

impl RtAudioEffect {
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

    pub fn apply_ui_style(egui_context: &egui::Context) {
        egui_context.style_mut(|x| {
            x.text_styles.insert(
                TextStyle::Body,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Button,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
        });
    }

    pub fn update_ui(&mut self) {
        self.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_context
            .begin_frame(self.egui_input_state.input.take());

        egui::Window::new("Analyzer parameters").show(&self.egui_context, |ui: &mut egui::Ui| {
            ui.horizontal(|ui| {
                if let Some(constant) = number_input::<f32>(
                    ui,
                    "Averaging Constant:",
                    &mut self.averaging_constant_value,
                ) {
                    self.audio_analyzer.set_averaging_constant(constant);
                };
            });
            ui.horizontal(|ui| {
                if let Some(fft_length) =
                    number_input::<u32>(ui, "FFT length:", &mut self.fft_length_value)
                {
                    self.audio_analyzer.set_fft_length(fft_length);
                };
            });
        });

        egui::TopBottomPanel::top("Top").show(&self.egui_context, |ui| {
            ui.menu_button("File", |ui| {
                {
                    let _ = ui.button("test 1");
                }
                ui.separator();
                {
                    let _ = ui.button("test 2");
                }
            });
        });
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
        //Since we are custom drawing an OpenGL Triangle we don't need egui to clear the background.

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
                    println!("{:?}", event);
                    egui_glfw::handle_event(event, &mut self.egui_input_state);
                }
            }
        }
    }
}
