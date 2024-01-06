use super::{RtAudioEffect, UiContext};
use crate::{
    audio::AudioAnalyzer,
    glfw_egui::{egui_glfw, glfw_painter},
    ui_helpers::ui_helpers::number_input,
};
use egui::{self, CollapsingHeader, FontId, Pos2, Rect, TextStyle, Vec2};
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
            x.text_styles.insert(
                TextStyle::Heading,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
        });
    }

    pub fn update_ui(&mut self) {
        self.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_context
            .begin_frame(self.egui_input_state.input.take());

        egui::Window::new("Analyzer parameters").show(&self.egui_context, |ui: &mut egui::Ui| {
            RtAudioEffect::show_stream_parameters(&self.audio_analyzer, ui);
            ui.separator();

            RtAudioEffect::show_fft_parameters(&self.audio_analyzer, ui);
            ui.separator();

            RtAudioEffect::show_parameter_editor(
                &mut self.ui_context,
                &mut self.audio_analyzer,
                ui,
            );
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

    fn show_stream_parameters(audio_analyzer: &AudioAnalyzer, ui: &mut egui::Ui) {
        CollapsingHeader::new("Stream parameters")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Samples rate:");
                    ui.label(audio_analyzer.host.sample_rate.to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("Number of channels:");
                    ui.label(audio_analyzer.host.channels.to_string());
                })
            });
    }

    fn show_fft_parameters(audio_analyzer: &AudioAnalyzer, ui: &mut egui::Ui) {
        CollapsingHeader::new("FFT parameters")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Spectrum width:");
                    ui.label(audio_analyzer.host.spectrum_width.to_string());
                });
                ui.horizontal(|ui| {
                    ui.label("Averaging constant:");
                    ui.label(audio_analyzer.host.get_averaging_constant().to_string());
                })
            });
    }

    fn show_parameter_editor(
        ui_context: &mut UiContext,
        audio_analyzer: &mut AudioAnalyzer,
        ui: &mut egui::Ui,
    ) {
        ui.horizontal(|ui| ui.centered_and_justified(|ui| ui.label("Parameter editor")));
        ui.horizontal(|ui| {
            if let Some(constant) = number_input::<f32>(
                ui,
                "Averaging Constant:",
                &mut ui_context.averaging_constant_value,
            ) {
                audio_analyzer.set_averaging_constant(constant);
            };
        });
        ui.horizontal(|ui| {
            if let Some(fft_length) =
                number_input::<u32>(ui, "FFT length:", &mut ui_context.fft_length_value)
            {
                audio_analyzer.set_fft_length(fft_length);
            };
        });
    }
}
