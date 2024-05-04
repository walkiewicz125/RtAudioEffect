use std::sync::{Arc, Mutex};

use egui::{Align, CollapsingHeader, Layout, Separator, Ui, Vec2, Widget};

use crate::{audio::audio_stream::AudioStream, audio_analyzer::AudioAnalyzysProvider};

use super::{
    helpers::{add_columns, add_rows},
    plot::BarSpectrumRenderer,
    spectrum_renderer_widget::SprectrumRendererWidget,
};

pub struct CentralPanel {
    audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
    audio_stream: Arc<Mutex<AudioStream>>,
    spectrum_left: SprectrumRendererWidget,
    spectrum_right: SprectrumRendererWidget,
}

impl CentralPanel {
    pub fn build(
        audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        audio_stream: Arc<Mutex<AudioStream>>,
        spectrum_renderer_left: Arc<Mutex<BarSpectrumRenderer>>,
        spectrum_renderer_right: Arc<Mutex<BarSpectrumRenderer>>,
    ) -> Self {
        Self {
            audio_analyzer,
            audio_stream,
            spectrum_left: SprectrumRendererWidget {
                renderer: spectrum_renderer_left,
            },
            spectrum_right: SprectrumRendererWidget {
                renderer: spectrum_renderer_right,
            },
        }
    }
}

impl Widget for CentralPanel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let stream_parameters = self.audio_stream.lock().unwrap().get_parameters();
        let analyzer_parameters = self
            .audio_analyzer
            .lock()
            .unwrap()
            .get_analyzer_parameters();

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
                        ui.label(analyzer_parameters.refresh_time.as_millis().to_string() + " ms");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Spectrogram durations:");
                        ui.label(
                            analyzer_parameters
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
                        ui.label(analyzer_parameters.spectrum_width.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Refresh time in samples:");
                        ui.label(analyzer_parameters.refresh_time_in_samples.to_string());
                    });
                });
        };

        let draw_stream_controls = |ui: &mut Ui| {
            ui.strong("Stream control:");
            ui.columns(2, |uis| {
                if uis[0].button("Start").clicked() {
                    self.audio_stream.lock().unwrap().start();
                }
                if uis[1].button("Stop").clicked() {
                    self.audio_stream.lock().unwrap().stop();
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

        ui.horizontal_top(|ui| {
            let response = ui.allocate_ui_with_layout(
                Vec2 {
                    x: 250.0,
                    y: ui.available_height(),
                },
                Layout::top_down(Align::LEFT),
                draw_parameters_and_control_panel,
            );

            ui.add(Separator::default().vertical());
            add_columns(ui, 2, |ui| {
                add_rows(&mut ui[0], 2, |ui| {
                    ui[0].add(self.spectrum_left);
                    ui[1].add(self.spectrum_right);
                });
                add_rows(&mut ui[1], 2, |ui| {
                    // ui[0].add(self.spectrogram_left);
                    // ui[1].add(self.spectrogram_right);
                });
            });
            response
        })
        .response
    }
}
