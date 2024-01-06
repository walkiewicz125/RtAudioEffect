use glfw::{Context, Glfw, WindowEvent};
use std::{sync::mpsc::Receiver, time::Instant};

use crate::{
    audio::AudioAnalyzer,
    glfw_egui::{egui_glfw, glfw_painter},
    plot::BarSpectrumRenderer,
};

mod helpers;
mod user_interface;
struct UiContext {
    averaging_constant_value: String,
    fft_length_value: String,
}
pub struct RtAudioEffect {
    glfw_context: Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    painter: glfw_painter::Painter,
    egui_context: egui::Context,
    egui_input_state: egui_glfw::EguiInputState,
    bar_spectrum_renderer: BarSpectrumRenderer,
    start_time: Instant,
    ui_context: UiContext,
    audio_analyzer: AudioAnalyzer,
}

impl RtAudioEffect {
    pub fn new(resolution: (u32, u32)) -> Self {
        let mut glfw_context = RtAudioEffect::create_glfx_context();
        let (mut window, events) = RtAudioEffect::create_window(&mut glfw_context, resolution);
        let (painter, egui_context, egui_input_state) = RtAudioEffect::initialize_egui(&mut window);

        let mut bar_spectrum_renderer = BarSpectrumRenderer::new().with_view(resolution);

        let mut spectrum: Vec<f32> = vec![0.0; 1024];

        for i in 0..1024 {
            spectrum[i] = i as f32 / 1024.0;
        }

        bar_spectrum_renderer.set_spectrum(spectrum.as_slice());

        RtAudioEffect::apply_ui_style(&egui_context);

        let size = 48000 / 10;
        let audio_analyzer = AudioAnalyzer::new_default_loopback(size as usize)
            .expect("Failed to create default loopback stream");

        RtAudioEffect {
            glfw_context,
            window,
            events,
            painter,
            egui_context,
            egui_input_state,
            bar_spectrum_renderer,
            start_time: Instant::now(),
            ui_context: UiContext {
                averaging_constant_value: audio_analyzer.get_averaging_constant().to_string(),
                fft_length_value: audio_analyzer.get_fft_length().to_string(),
            },
            audio_analyzer,
        }
    }

    pub fn run(&mut self) {
        self.audio_analyzer.start();

        while !self.window.should_close() {
            unsafe {
                gl::ClearColor(0.455, 0.302, 0.663, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            let magnitude: Vec<f32> = self.audio_analyzer.get_last_left_channel_mean_spectrum();
            self.bar_spectrum_renderer.set_spectrum(&magnitude);
            self.bar_spectrum_renderer.set_style(1);
            self.bar_spectrum_renderer.render();

            let magnitude: Vec<f32> = self.audio_analyzer.get_last_left_channel_spectrum();
            self.bar_spectrum_renderer.set_spectrum(&magnitude);
            self.bar_spectrum_renderer.set_style(0);
            self.bar_spectrum_renderer.render();

            self.update_ui();

            self.window.swap_buffers();
            self.glfw_context.poll_events();
        }
    }
}

impl Default for RtAudioEffect {
    fn default() -> Self {
        Self::new((1, 1))
    }
}
