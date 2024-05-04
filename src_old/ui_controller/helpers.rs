use glfw::{Context, Glfw, WindowEvent};
use std::sync::mpsc::Receiver;

pub fn create_glfx_context() -> Glfw {
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

pub fn create_window(
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
