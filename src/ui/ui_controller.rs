use std::sync::{Arc, Mutex};

use egui::{FontId, TextStyle, TextureId};

use crate::{audio::audio_stream::AudioStream, audio_analyzer::AudioAnalyzysProvider};

use super::{
    central_panel::{CentralPanel, HeatMapImage},
    plot::spectrum::{
        spectrogram_renderer::SpectrogramRenderer, spectrum_renderer::SpectrumRenderer,
    },
};

pub struct UiController {
    audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
    audio_stream: Arc<Mutex<AudioStream>>,
    spectrum_renderer_left: Arc<Mutex<SpectrumRenderer>>,
    spectrum_renderer_right: Arc<Mutex<SpectrumRenderer>>,
    spectrogram_renderer_left: Arc<Mutex<SpectrogramRenderer>>,
    spectrogram_renderer_right: Arc<Mutex<SpectrogramRenderer>>,
    heat_map: HeatMapImage,
    auto_range: bool,
}

impl UiController {
    pub fn new(
        audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        audio_stream: Arc<Mutex<AudioStream>>,
        heat_map: HeatMapImage,
    ) -> Self {
        Self {
            audio_analyzer,
            audio_stream,
            spectrum_renderer_left: Arc::new(Mutex::new(SpectrumRenderer::new())),
            spectrum_renderer_right: Arc::new(Mutex::new(SpectrumRenderer::new())),
            spectrogram_renderer_left: Arc::new(Mutex::new(SpectrogramRenderer::new())),
            spectrogram_renderer_right: Arc::new(Mutex::new(SpectrogramRenderer::new())),
            heat_map,
            auto_range: true,
        }
    }

    pub fn update_data(&self) {
        self.spectrum_renderer_left
            .lock()
            .unwrap()
            .set_spectrum(&self.audio_analyzer.lock().unwrap().get_latest_spectrum()[0]);
        self.spectrum_renderer_right
            .lock()
            .unwrap()
            .set_spectrum(&self.audio_analyzer.lock().unwrap().get_latest_spectrum()[1]);

        self.spectrogram_renderer_left.lock().unwrap().buffer_data(
            self.audio_analyzer
                .lock()
                .unwrap()
                .get_spectrogram_for_channel(0),
        );
        self.spectrogram_renderer_right.lock().unwrap().buffer_data(
            self.audio_analyzer
                .lock()
                .unwrap()
                .get_spectrogram_for_channel(1),
        );
    }

    pub fn get_central_panel(&self) -> CentralPanel {
        CentralPanel::build(
            self.audio_analyzer.clone(),
            self.audio_stream.clone(),
            self.spectrum_renderer_left.clone(),
            self.spectrum_renderer_right.clone(),
            self.spectrogram_renderer_left.clone(),
            self.spectrogram_renderer_right.clone(),
            self.heat_map.clone(),
            self.auto_range,
        )
    }

    pub fn set_text_styles(&self, egui_context: &egui::Context, font_size: f32) {
        egui_context.style_mut(|x| {
            x.text_styles.insert(
                TextStyle::Body,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Button,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Heading,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(font_size, egui::FontFamily::Proportional),
            );
            x.text_styles.insert(
                TextStyle::Small,
                FontId::new(font_size * 0.8, egui::FontFamily::Proportional),
            );
        });
    }
}
