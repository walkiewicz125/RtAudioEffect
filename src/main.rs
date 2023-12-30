mod glfw_egui;
mod plot;

mod app;
use app::RtAudioEffect;
mod audio_host;
use audio_host::*;

const SCREEN_WIDTH: u32 = 1920;
const SCREEN_HEIGHT: u32 = 1080;
const DEFAULT_RESOLUTION: (u32, u32) = (SCREEN_WIDTH, SCREEN_HEIGHT);

fn main() {
    println!("Hello, world!");

    let audio =
        AudioSource::new_default_loopback().expect("Failed to create default loopback stream");
    audio.start();

    let mut app_context = RtAudioEffect::new(DEFAULT_RESOLUTION);
    app_context.run();

    println!("Goodbye, world!");
}
