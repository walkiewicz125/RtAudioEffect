use std::{
    borrow::BorrowMut,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use egui::{
    epaint::{TessellationOptions, TextureManager},
    load::SizedTexture,
    vec2, Align, CollapsingHeader, Color32, ColorImage, Context, ImageData, ImageSource, Layout,
    Pos2, Rect, Response, Rounding, Sense, Separator, Slider, TextureId, TextureOptions, Ui, Vec2,
    Widget,
};
use log::info;

use crate::{audio::audio_stream::AudioStream, audio_analyzer::AudioAnalyzysProvider};

use super::{
    helpers::{self, add_columns, add_rows},
    plot::spectrum::{
        spectrogram_renderer::SpectrogramRenderer,
        spectrogram_renderer_widget::SpectrogramRendererWidget,
        spectrum_renderer::SpectrumRenderer, spectrum_renderer_widget::SprectrumRendererWidget,
    },
};

pub struct CentralPanel {
    audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
    audio_stream: Arc<Mutex<AudioStream>>,
    spectrum_left: SprectrumRendererWidget,
    spectrum_right: SprectrumRendererWidget,
    spectrogram_left: SpectrogramRendererWidget,
    spectrogram_right: SpectrogramRendererWidget,
    heat_map: HeatMapImage,
    auto_range: bool,
    fps: f32,
}

impl CentralPanel {
    pub fn build(
        audio_analyzer: Arc<Mutex<dyn AudioAnalyzysProvider>>,
        audio_stream: Arc<Mutex<AudioStream>>,
        spectrum_renderer_left: Arc<Mutex<SpectrumRenderer>>,
        spectrum_renderer_right: Arc<Mutex<SpectrumRenderer>>,
        spectrogram_renderer_left: Arc<Mutex<SpectrogramRenderer>>,
        spectrogram_renderer_right: Arc<Mutex<SpectrogramRenderer>>,
        heat_map: HeatMapImage,
        auto_range: bool,
        fps: f32,
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
            spectrogram_left: SpectrogramRendererWidget {
                renderer: spectrogram_renderer_left,
            },
            spectrogram_right: SpectrogramRendererWidget {
                renderer: spectrogram_renderer_right,
            },
            heat_map,
            auto_range,
            fps,
        }
    }
}

impl Widget for CentralPanel {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
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
            ui.separator();

            CollapsingHeader::new("Magnitude colors")
                .default_open(true)
                .show(ui, |ui| {
                    ui.group(|ui| {
                        ui.with_layout(
                            Layout::left_to_right(Align::Min)
                                .with_main_align(Align::Min)
                                .with_main_justify(true),
                            |ui| {
                                ui.label("min");
                                ui.label("max");
                            },
                        );
                        ui.add(RangeWidget::new_horizontal(self.heat_map));
                    })
                });
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.label("FPS:");
                ui.label(self.fps.to_string());
            });
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

            add_columns(ui, 2, |ui| {
                ui[0].add_sized(ui[0].available_size() / vec2(1.0, 3.0), self.spectrum_left);
                ui[0].add(self.spectrogram_left);

                ui[1].add_sized(ui[1].available_size() / vec2(1.0, 3.0), self.spectrum_right);
                ui[1].add(self.spectrogram_right);
            });
            response
        })
        .response
    }
}

#[derive(Clone)]
pub struct HeatMapImage {
    texture_horizontal: egui::TextureHandle,
    texture_vertical: egui::TextureHandle,
}

impl HeatMapImage {
    const RESOLUTION: usize = 1000;

    pub fn new(ctx: &Context) -> Self {
        let mut srgba = vec![Color32::TRANSPARENT; 1000];
        for i in (0..1000).rev() {
            let value = i as f32 / 1000.0;
            let color = Self::color_map(value);
            srgba[i] = color;
        }
        let texture_vertical = ctx.load_texture(
            "heat_map_vertical",
            ImageData::Color(Arc::new(ColorImage {
                size: [1, Self::RESOLUTION],
                pixels: srgba.iter().rev().cloned().collect(),
            })),
            TextureOptions::LINEAR,
        );
        let texture_horizontal = ctx.load_texture(
            "heat_map_horizontal",
            ImageData::Color(Arc::new(ColorImage {
                size: [Self::RESOLUTION, 1],
                pixels: srgba,
            })),
            TextureOptions::LINEAR,
        );

        Self {
            texture_horizontal,
            texture_vertical,
        }
    }

    fn mix(color1: Color32, color2: Color32, value: f32) -> Color32 {
        let r = color1.r() as f32 * (1.0 - value) + color2.r() as f32 * value;
        let g = color1.g() as f32 * (1.0 - value) + color2.g() as f32 * value;
        let b = color1.b() as f32 * (1.0 - value) + color2.b() as f32 * value;
        let a = color1.a() as f32 * (1.0 - value) + color2.a() as f32 * value;

        Color32::from_rgba_premultiplied(r as u8, g as u8, b as u8, a as u8)
    }

    fn color_map(value: f32) -> Color32 {
        let color1 = Color32::from_rgba_premultiplied(0, 0, 255, 0);
        let color2 = Color32::from_rgba_premultiplied(0, 255, 255, 255);
        let color3 = Color32::from_rgba_premultiplied(0, 255, 0, 255);
        let color4 = Color32::from_rgba_premultiplied(255, 255, 0, 255);
        let color5 = Color32::from_rgba_premultiplied(255, 0, 0, 255);

        if value < 0.25 {
            Self::mix(color1, color2, value * 4.0)
        } else if value < 0.5 {
            Self::mix(color2, color3, (value - 0.25) * 4.0)
        } else if value < 0.75 {
            Self::mix(color3, color4, (value - 0.5) * 4.0)
        } else {
            Self::mix(color4, color5, (value - 0.75) * 4.0)
        }
    }
}

enum RangeWidgetOrientation {
    Vertical,
    Horizontal,
}

struct RangeWidget {
    heat_map: HeatMapImage,
    orientation: RangeWidgetOrientation,
}

impl RangeWidget {
    const THICKNESS: f32 = 10.0;

    pub fn new_vertical(heat_map: HeatMapImage) -> Self {
        Self {
            heat_map,
            orientation: RangeWidgetOrientation::Vertical,
        }
    }

    pub fn new_horizontal(heat_map: HeatMapImage) -> Self {
        Self {
            heat_map,
            orientation: RangeWidgetOrientation::Horizontal,
        }
    }
}

impl Widget for RangeWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self.orientation {
            RangeWidgetOrientation::Vertical => {
                let response = ui.allocate_response(
                    Vec2::new(Self::THICKNESS, ui.available_height()),
                    Sense::click_and_drag(),
                );
                if ui.is_rect_visible(response.rect) {
                    egui::Image::new(SizedTexture::from_handle(&self.heat_map.texture_vertical))
                        .paint_at(ui, response.rect);
                }

                response
            }
            RangeWidgetOrientation::Horizontal => {
                let response = ui.allocate_response(
                    Vec2::new(ui.available_width(), Self::THICKNESS),
                    Sense::click_and_drag(),
                );
                if ui.is_rect_visible(response.rect) {
                    egui::Image::new(SizedTexture::from_handle(&self.heat_map.texture_horizontal))
                        .paint_at(ui, response.rect);
                }

                response
            }
        }
    }
}
