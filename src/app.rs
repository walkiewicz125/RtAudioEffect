use glam::{vec4, Vec2};
use glfw::{Context, Glfw, WindowEvent};
use std::{sync::mpsc::Receiver, time::Instant};

use crate::{
    audio::AudioAnalyzer,
    glfw_egui::{egui_glfw, glfw_painter},
    plot::BarSpectrumRenderer,
    plot::LinesRenderer,
    plot::TextureRenderTarget,
};

mod helpers;
mod user_interface;
struct UiController {
    averaging_constant_value: String,
    fft_length_value: String,
    spectrum_texture: egui::TextureId,
}

struct RtAudioEffectContext {
    glfw_context: Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    painter: glfw_painter::Painter,
    egui_context: egui::Context,
    egui_input_state: egui_glfw::EguiInputState,
}

struct Renderers {
    spectrum_renderer: BarSpectrumRenderer,
    spectrum_render_target: TextureRenderTarget,
    lines_renderer: LinesRenderer,
    spectrum_resolution: (u32, u32),
}

impl Renderers {
    fn new(resolution: (u32, u32)) -> Renderers {
        let mut spectrum_renderer = BarSpectrumRenderer::new().with_view(resolution);
        let mut spectrum: Vec<f32> = vec![0.0; 1024];
        for i in 0..1024 {
            spectrum[i] = i as f32 / 1024.0;
        }
        spectrum_renderer.set_spectrum(spectrum.as_slice());

        let spectrum_render_target = TextureRenderTarget::new(resolution);

        let mut lines_renderer = LinesRenderer::new().with_view(resolution);
        let mut lines_points = vec![];
        lines_points.push(Vec2 { x: 0.0, y: 0.0 });
        lines_points.push(Vec2 {
            x: 500.0,
            y: -900.0,
        });
        lines_points.push(Vec2 { x: 500., y: 900.0 });
        lines_renderer.set_line_strip_open(&lines_points);
        lines_renderer.set_line_width(200.0);
        lines_renderer.set_line_color(vec4(1.0, 0.2, 0.2, 1.0));

        spectrum_renderer.set_style(0);
        spectrum_renderer.flip_vertically(true);
        Renderers {
            spectrum_renderer,
            spectrum_render_target,
            lines_renderer,
            spectrum_resolution: resolution,
        }
    }

    fn set_resolution_of_spectrum(&mut self, available_size: egui::Vec2) {
        let new_resolution = (available_size.x as u32, available_size.y as u32);
        if new_resolution != self.spectrum_resolution {
            self.spectrum_resolution = new_resolution;
            self.spectrum_renderer.set_view(self.spectrum_resolution);
            self.spectrum_render_target
                .set_resolution(self.spectrum_resolution);
        }
    }
}

pub struct RtAudioEffect {
    context: RtAudioEffectContext,
    renderers: Renderers,
    ui_controller: UiController,
    audio_analyzer: AudioAnalyzer,
    start_time: Instant,
}

/*
AudioProcessing
Rendering
Context handling
*/
impl RtAudioEffect {
    pub fn new(resolution: (u32, u32)) -> Self {
        let mut glfw_context = RtAudioEffect::create_glfx_context();
        let (mut window, events) = RtAudioEffect::create_window(&mut glfw_context, resolution);
        let (mut painter, egui_context, egui_input_state) =
            RtAudioEffect::initialize_egui(&mut window);

        RtAudioEffect::apply_ui_style(&egui_context);

        let size = 48000 / 10;
        let audio_analyzer = AudioAnalyzer::new_default_loopback(size as usize)
            .expect("Failed to create default loopback stream");

        let renderers = Renderers::new(resolution);

        let spectrum_texture_id: egui::TextureId =
            painter.new_opengl_texture(renderers.spectrum_render_target.get_texture_id());
        RtAudioEffect {
            context: RtAudioEffectContext {
                glfw_context,
                window,
                events,
                painter,
                egui_context,
                egui_input_state,
            },
            ui_controller: UiController {
                averaging_constant_value: audio_analyzer.get_averaging_constant().to_string(),
                fft_length_value: audio_analyzer.get_fft_length().to_string(),
                spectrum_texture: spectrum_texture_id,
            },
            audio_analyzer,
            renderers,
            start_time: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        self.audio_analyzer.start();

        while !self.context.window.should_close() {
            unsafe {
                gl::ClearColor(0.455, 0.302, 0.663, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            let magnitude: Vec<f32> = self.audio_analyzer.get_last_left_channel_mean_spectrum();
            self.renderers.spectrum_renderer.set_spectrum(&magnitude);
            self.renderers
                .spectrum_render_target
                .render(&self.renderers.spectrum_renderer);

            self.update_ui();

            self.context.window.swap_buffers();
            self.context.glfw_context.poll_events();
        }
    }
}

impl Default for RtAudioEffect {
    fn default() -> Self {
        Self::new((1, 1))
    }
}
