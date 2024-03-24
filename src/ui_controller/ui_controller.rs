use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Instant,
};

use egui::{
    load::SizedTexture, pos2, vec2, Align, CollapsingHeader, FontId, Image, InnerResponse, Layout,
    Rect, Separator, TextStyle, TextureId, Ui, Vec2,
};
use glfw::{Context, Glfw, WindowEvent};

use crate::{
    audio_processor::AudioAnalyzysProvider,
    glfw_egui::{egui_glfw, glfw_painter},
    plot::{BarSpectrumRenderer, TextureRenderTarget},
};

use super::{add_rows, egui_helpers, helpers};

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

    fn central_panel<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        egui::CentralPanel::default().show(&self.egui_context, add_contents)
    }

    fn top_panel<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        egui::TopBottomPanel::top("Top").show(&self.egui_context, add_contents)
    }
}

struct UiSpectrumRenderer {
    spectrum_renderer: BarSpectrumRenderer,
    spectrum_texture_renderer: TextureRenderTarget,
    spectrum_tex_id: TextureId,
}

impl UiSpectrumRenderer {
    fn new(ui_window: &mut UiWindow) -> UiSpectrumRenderer {
        let mut spectrum_renderer = BarSpectrumRenderer::new();
        let spectrum_texture_renderer = TextureRenderTarget::new((1, 1));

        let spectrum_tex_id = ui_window
            .painter
            .new_opengl_texture(spectrum_texture_renderer.get_texture_id());
        spectrum_renderer.flip_vertically(true);

        UiSpectrumRenderer {
            spectrum_renderer,
            spectrum_texture_renderer,
            spectrum_tex_id,
        }
    }
    fn update_spectrum(
        &mut self,
        spectrum_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        channel_number: usize,
    ) {
        let spectrum =
            spectrum_provider.lock().unwrap().get_latest_spectrum()[channel_number].clone();
        self.spectrum_renderer.set_spectrum(spectrum.as_slice());
        self.spectrum_renderer.set_style(0)
    }

    fn render(&mut self, ui: &mut Ui) {
        unsafe {
            gl::ClearColor(0.455, 0.302, 0.663, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        let convert_vec2_to_tuple = |resolution: Vec2| (resolution.x as u32, resolution.y as u32);
        let resolution = convert_vec2_to_tuple(ui.available_size());
        self.spectrum_renderer.set_view(resolution);
        self.spectrum_texture_renderer.set_resolution(resolution);
        self.spectrum_texture_renderer
            .render(&self.spectrum_renderer);

        ui.add(Image::from_texture(SizedTexture {
            id: self.spectrum_tex_id,
            size: ui.available_size(),
        }));
    }
}

pub struct UiController {
    window: UiWindow,
    audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
    spectrum_texture_renderer_left: UiSpectrumRenderer,
    spectrum_texture_renderer_right: UiSpectrumRenderer,
}

impl UiController {
    pub fn new(
        audio_analyzis_provider: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        resolution: Resolution,
    ) -> UiController {
        let mut window = UiWindow::new(resolution);
        let spectrum_texture_renderer_left = UiSpectrumRenderer::new(&mut window);
        let spectrum_texture_renderer_right = UiSpectrumRenderer::new(&mut window);

        UiController {
            window,
            audio_analyzis_provider,
            spectrum_texture_renderer_left,
            spectrum_texture_renderer_right,
        }
    }

    pub fn is_closing(&self) -> bool {
        self.window.should_close()
    }

    fn draw_top_panel(&mut self) {
        self.window.top_panel(|ui| {
            ui.menu_button("File", |ui| {
                let _ = ui.button("test 1");
                ui.separator();
                let _ = ui.button("test 1");
            })
        });
    }

    fn draw_central_panel(&mut self) {
        let analyzer_parmeters = self
            .audio_analyzis_provider
            .lock()
            .unwrap()
            .get_analyzer_parameters();
        let stream_parameters = self
            .audio_analyzis_provider
            .lock()
            .unwrap()
            .get_stream_parameters();

        let draw_stream_parameters = |ui: &mut Ui| {
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
        };

        let draw_analyzer_parameters = |ui: &mut Ui| {
            CollapsingHeader::new("Analyzer parameters")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Refresh time:");
                        ui.label(analyzer_parmeters.refresh_time.as_millis().to_string() + " ms");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Spectrogram durations:");
                        ui.label(
                            analyzer_parmeters
                                .spectrogram_duration
                                .as_secs_f32()
                                .to_string()
                                + " s",
                        );
                    });
                });
        };

        let draw_fft_parameters = |ui: &mut Ui| {
            CollapsingHeader::new("FFT parameters")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Spectrum width:");
                        ui.label(analyzer_parmeters.spectrum_width.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Refresh time in samples:");
                        ui.label(analyzer_parmeters.refresh_time_in_samples.to_string());
                    });
                });
        };

        let draw_stream_controls = |ui: &mut Ui| {
            ui.strong("Stream control:");
            ui.columns(2, |uis| {
                if uis[0].button("Start").clicked() {
                    // TODO: Start stream
                }
                if uis[1].button("Stop").clicked() {
                    // TODO: Stop stream
                }
            })
        };

        let draw_parameters_and_control_panel = |ui: &mut Ui| {
            draw_stream_parameters(ui);
            ui.separator();

            draw_analyzer_parameters(ui);
            ui.separator();

            draw_fft_parameters(ui);
            ui.separator();

            draw_stream_controls(ui);
        };

        let mut draw_spectrum = |ui: &mut Ui| {
            add_rows(ui, 2, |uis| {
                self.spectrum_texture_renderer_left.render(&mut uis[0]);
                self.spectrum_texture_renderer_left.render(&mut uis[1]);
            });
        };

        self.window.central_panel(|ui| {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2 {
                        x: 250.0,
                        y: ui.available_height(),
                    },
                    Layout::top_down(Align::LEFT),
                    draw_parameters_and_control_panel,
                );

                ui.add(Separator::default().vertical());

                draw_spectrum(ui);
            })
        });
    }

    pub fn render(&mut self) {
        if !self.window.should_close() {
            self.spectrum_texture_renderer_left
                .update_spectrum(self.audio_analyzis_provider.clone(), 0);
            self.spectrum_texture_renderer_right
                .update_spectrum(self.audio_analyzis_provider.clone(), 1);

            self.window.begin_frame();

            self.draw_top_panel();
            self.draw_central_panel();

            self.window.finalize_frame();
        }
    }
}
