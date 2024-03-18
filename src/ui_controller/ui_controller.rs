use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Instant,
};

use egui::{
    load::SizedTexture, Align, CollapsingHeader, FontId, Image, Layout, Separator, TextStyle, Vec2,
};
use glfw::{Context, Glfw, WindowEvent};

use crate::{
    audio::StreamParameters,
    audio_analyzer::{AnalyzerParameters, SpectrumAnalyzer},
    glfw_egui::{egui_glfw, glfw_painter},
};

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
fn show_stream_parameters(stream_parameters: Arc<StreamParameters>, ui: &mut egui::Ui) {
    CollapsingHeader::new("Stream parameters")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Sample rate:");
                ui.label(stream_parameters.sample_rate.to_string());
            });
            ui.horizontal(|ui| {
                ui.label("Number of channels:");
                ui.label(stream_parameters.channels.to_string());
            })
        });
}

fn show_spectrum_parameters(spectrum_parameters: Arc<AnalyzerParameters>, ui: &mut egui::Ui) {
    CollapsingHeader::new("FFT parameters")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Spectrum width:");
                ui.label(spectrum_parameters.spectrum_width.to_string());
            });
            ui.horizontal(|ui| {
                ui.label("Refresh time in samples:");
                ui.label(spectrum_parameters.refresh_time_in_samples.to_string());
            });
        });
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

    fn draw_top_panel(&mut self) {
        egui::TopBottomPanel::top("Top").show(&self.egui_context, |ui| {
            ui.menu_button("File", |ui| {
                let _ = ui.button("test 1");
                ui.separator();
                let _ = ui.button("test 1");
            })
        });
    }

    fn begin_frame(&mut self) {
        self.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_context
            .begin_frame(self.egui_input_state.input.take());
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

        self.window.swap_buffers();
        self.glfw_context.poll_events();
    }

    fn draw_central_panel(
        &mut self,
        audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
    ) {
        egui::CentralPanel::default().show(&self.egui_context, |ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2 {
                        x: 250.0,
                        y: ui.available_height(),
                    },
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // TODO: show stream parameters
                        show_stream_parameters(
                            audio_analyzis_provider
                                .lock()
                                .unwrap()
                                .get_stream_parameters(),
                            ui,
                        );
                        ui.separator();
                        show_spectrum_parameters(
                            audio_analyzis_provider
                                .lock()
                                .unwrap()
                                .get_analyzer_parameters(),
                            ui,
                        );
                        ui.separator();
                        // TODO: show spectrum editor
                        ui.separator();

                        ui.strong("Stream control:");
                        ui.columns(2, |uis| {
                            if uis[0].button("Start").clicked() {
                                // TODO: Start stream
                            }
                            if uis[1].button("Stop").clicked() {
                                // TODO: Stop stream
                            }
                        })
                    },
                );

                // Render spectrum
                ui.add(Separator::default().vertical());
                // TODO:
                // ui.add(Image::from_texture(SizedTexture {
                //     id: self.ui_controller.spectrum_texture,
                //     size: ui.available_size(),
                // }));
            })
        });
    }
}

pub struct UiController {
    window: UiWindow,
    audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
}
pub trait AudioAnalyzysProvider {
    fn get_stream_parameters(&self) -> Arc<StreamParameters>;
    fn get_analyzer_parameters(&self) -> Arc<AnalyzerParameters>;
}

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
        if !self.window.should_close() {
            self.window.begin_frame();
            self.window.draw_top_panel();
            self.window
                .draw_central_panel(self.audio_analyzis_provider.clone());
            self.window.finalize_frame();
        }
    }

    pub fn is_closing(&self) -> bool {
        self.window.should_close()
    }
}
