use std::{sync::mpsc::Receiver, time::Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use glam::Mat4;
use glfw::{Context, Glfw, WindowEvent};

mod plot;
use plot::BarSpectrumRenderer;

mod glfw_egui;
use egui::{panel::Side, vec2, FontId, Id, Pos2, Rect, TextStyle};
use glfw_egui::{egui_glfw, glfw_painter};

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;

fn main() {
    println!("Hello, world!");

    let mut app_context = AppContext::new();
    app_context.run();

    println!("Goodbye, world!");
}

struct AppContext {
    glfw_context: Glfw,
    window: glfw::Window,
    events: Receiver<(f64, WindowEvent)>,
    painter: glfw_painter::Painter,
    egui_context: egui::Context,
    egui_input_state: egui_glfw::EguiInputState,
    bar_spectrum_renderer: BarSpectrumRenderer,
}

impl AppContext {
    pub fn new() -> Self {
        let mut glfw_context = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw_context.window_hint(glfw::WindowHint::ContextVersion(4, 3));
        glfw_context.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw_context.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw_context.window_hint(glfw::WindowHint::Resizable(true));
        glfw_context.window_hint(glfw::WindowHint::Samples(Some(8)));

        let (mut window, events) = AppContext::create_window(&mut glfw_context);

        window.set_char_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_key_polling(true);
        window.set_mouse_button_polling(true);
        window.set_size_polling(true);
        window.make_current();

        gl::load_with(|s| window.get_proc_address(s));
        unsafe { gl::Enable(gl::MULTISAMPLE) };
        let (painter, egui_context, egui_input_state) = AppContext::initialize_egui(&mut window);

        let resolution = (SCREEN_WIDTH, SCREEN_HEIGHT);
        let mut bar_spectrum_renderer = BarSpectrumRenderer::new();

        let view_matrix = Mat4::orthographic_rh(
            0.0,
            resolution.0 as f32,
            0.0,
            resolution.1 as f32,
            -1.0,
            1.0,
        );
        bar_spectrum_renderer.set_view(view_matrix, resolution);
        let mut spectrum: Vec<f32> = vec![0.0; 1024];

        for i in 0..1024 {
            spectrum[i] = i as f32 / 1024.0;
        }

        bar_spectrum_renderer.set_spectrum(spectrum.as_slice());

        AppContext {
            glfw_context,
            window,
            events,
            painter,
            egui_context,
            egui_input_state,
            bar_spectrum_renderer,
        }
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();

        self.egui_context.style_mut(|x| {
            x.text_styles.insert(
                TextStyle::Body,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Button,
                FontId::new(20.0, egui::FontFamily::Proportional),
            );
        });

        while !self.window.should_close() {
            self.egui_input_state.input.time = Some(start_time.elapsed().as_secs_f64());
            self.egui_context
                .begin_frame(self.egui_input_state.input.take());

            unsafe {
                gl::ClearColor(0.455, 0.302, 0.663, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            self.bar_spectrum_renderer.render();

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
                egui_glfw::copy_to_clipboard(
                    &mut self.egui_input_state,
                    platform_output.copied_text,
                );
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
            self.window.swap_buffers();
            self.glfw_context.poll_events();
        }
    }

    fn create_window(glfw_context: &mut Glfw) -> (glfw::Window, Receiver<(f64, WindowEvent)>) {
        glfw_context
            .create_window(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                "Egui in GLFW!",
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window")
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
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

fn test_audio() {
    let available_hosts = cpal::available_hosts();

    if available_hosts.len() > 0 {
        let host = available_hosts[0];
        println!("selected host: {}", host.name());
        start_host(host);
    }
}

fn start_host(host_id: cpal::HostId) {
    let host = cpal::host_from_id(host_id).expect("failed to find Host");
    let device = host
        .default_output_device()
        .expect("failed to find output device");

    let config = device
        .default_output_config()
        .expect("failed to get default config");

    let err_fn = move |err| {
        eprintln!("an error occured on stream: {}", err);
    };

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    println!("Sample rate: {}, channels: {}", sample_rate, channels);
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| write_input_data::<f32>(data),
            err_fn,
            None,
        ),
        _ => todo!(),
    }
    .expect("failed to build inout stream");

    stream.play();

    while true {}
}

fn write_input_data<T: std::fmt::Display>(data: &[T]) {
    println!("len: {}", data.len());
}
