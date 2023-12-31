use std::{sync::mpsc::Receiver, time::Instant};

use egui::{panel::Side, vec2, FontId, Id, Pos2, Rect, TextStyle};
use glfw::{Context, Glfw, WindowEvent};
use rustfft::{num_complex::Complex, FftPlanner};

use crate::{
    audio::AudioSource,
    glfw_egui::{egui_glfw, glfw_painter},
    plot::BarSpectrumRenderer,
};

pub struct RtAudioEffect {
    glfw_context: Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    painter: glfw_painter::Painter,
    egui_context: egui::Context,
    egui_input_state: egui_glfw::EguiInputState,
    bar_spectrum_renderer: BarSpectrumRenderer,
    start_time: Instant,
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

        RtAudioEffect {
            glfw_context,
            window,
            events,
            painter,
            egui_context,
            egui_input_state,
            bar_spectrum_renderer,
            start_time: Instant::now(),
        }
    }

    pub fn run(&mut self) {
        let mut audio =
            AudioSource::new_default_loopback().expect("Failed to create default loopback stream");
        audio.start();

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(10240);

        while !self.window.should_close() {
            unsafe {
                gl::ClearColor(0.455, 0.302, 0.663, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            let last_data = audio.get_last_left_channel(10240);

            let mut data_in_complex: Vec<Complex<f32>> = last_data
                .iter()
                .map(|value| Complex {
                    re: value.clone(),
                    im: 0.0,
                })
                .collect();

            if data_in_complex.len() >= 10240 {
                fft.process(&mut data_in_complex);

                let magnitude: Vec<f32> = data_in_complex
                    .iter()
                    .map(|number| number.norm() / 10240.0)
                    .collect();

                self.bar_spectrum_renderer.set_spectrum(&magnitude);
                self.bar_spectrum_renderer.render();
            }

            self.update_ui();

            self.window.swap_buffers();
            self.glfw_context.poll_events();
        }
    }

    fn create_glfx_context() -> Glfw {
        let mut glfw_context = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        glfw_context.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw_context.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw_context.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw_context.window_hint(glfw::WindowHint::Resizable(true));
        glfw_context.window_hint(glfw::WindowHint::Samples(Some(8)));

        glfw_context
    }

    fn create_window(
        glfw_context: &mut Glfw,
        resolution: (u32, u32),
    ) -> (glfw::Window, Receiver<(f64, WindowEvent)>) {
        let (mut window, event_receiver) = glfw_context
            .create_window(
                resolution.0,
                resolution.1,
                "Egui in GLFW!",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window");

        window.set_char_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_size_polling(true);
        window.make_current();

        gl::load_with(|s| window.get_proc_address(s));
        unsafe { gl::Enable(gl::MULTISAMPLE) };

        (window, event_receiver)
    }

    fn initialize_egui(
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
                vec2(width as f32, height as f32) / native_pixels_per_point,
            )),
            ..Default::default() // todo: add pixels_per_point
        });

        (painter, egui_ctx, egui_input_state)
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
        });
    }

    fn update_ui(&mut self) {
        self.egui_input_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
        self.egui_context
            .begin_frame(self.egui_input_state.input.take());

        egui::SidePanel::new(Side::Right, Id::new("panel"))
            .show(&self.egui_context, |ui| ui.label("Test"));
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

impl Default for RtAudioEffect {
    fn default() -> Self {
        Self::new((1, 1))
    }
}
