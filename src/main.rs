mod glfw_egui;
mod plot;

mod app;
use app::RtAudioEffect;
mod audio;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: (u32, u32) = (SCREEN_WIDTH, SCREEN_HEIGHT);

fn main() {
    println!("Hello, world!");

    let mut app_context = RtAudioEffect::new(DEFAULT_RESOLUTION);
    app_context.run();

    println!("Goodbye, world!");
}
