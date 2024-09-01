use std::{mem::size_of, time::Duration};

use glam::{Mat4, Vec2, Vec3, Vec4};
use glamour::{Matrix4, Vector2};
use log::info;

use crate::{
    audio_analyzer::Spectrum,
    ui::plot::details::{
        shader_program::ShaderProgram, storage_buffer::StorageBufferArray,
        uniform_buffer::UniformBuffer, vertex_array::VertexArray,
        vertex_attribute::VertexAttribute, vertex_buffer::VertexBuffer,
    },
};

pub struct RenderConfig {
    pub projection: Matrix4<f32>,
    pub client_size: Vector2<f32>,
}

impl RenderConfig {
    fn new() -> Self {
        Self {
            projection: Matrix4::<f32>::IDENTITY,
            client_size: Vector2::new(0.0, 0.0),
        }
    }
}

pub struct DrawConfig {
    pub min_max: Vector2<f32>,
    pub bar_count: u32,
    pub scale: f32,
}

impl DrawConfig {
    fn new() -> Self {
        Self {
            min_max: Vector2::new(0.0, 0.0),
            bar_count: 0,
            scale: 1.0,
        }
    }
}

pub struct SpectrumRendererShader {
    shader: ShaderProgram,
    vertex_array: VertexArray,
    vertices_buffer: VertexBuffer<Vec2>,
    draw_config: UniformBuffer,
    render_config: UniformBuffer,
    magnitudes: StorageBufferArray<f32>,
}

impl SpectrumRendererShader {
    pub fn new(bar_segments: u32) -> Self {
        if bar_segments < 1 {
            panic!("BarPlotConfig.bar_segments must be greater than 0");
        }
        if bar_segments % 2 != 0 {
            panic!("BarPlotConfig.bar_segments must be even");
        }

        let shader = ShaderProgram::new_from_string(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)
            .expect("Failed to create SpectrumRenderer ShaderProgram");

        let vertex_array = VertexArray::new();
        let render_config = UniformBuffer::new_for::<RenderConfig>();
        let draw_config = UniformBuffer::new_for::<DrawConfig>();

        let vertices_buffer =
            Self::generate_vertices_buffer(&Self::generate_vertices(bar_segments));

        Self {
            shader,
            vertex_array,
            vertices_buffer,
            render_config,
            draw_config,
            magnitudes: StorageBufferArray::new(),
        }
    }

    pub fn set_magnitdes(&mut self, magnitudes: &[f32]) {
        self.magnitudes.store_array(magnitudes);

        let min = magnitudes.into_iter().cloned().reduce(f32::min).unwrap();
        let max = magnitudes.into_iter().cloned().reduce(f32::max).unwrap() * 0.9;
        let min_max = Vec2::new(min, max);
        let blend_func = (gl::CONSTANT_COLOR, gl::DST_ALPHA);
        let blend_color = Vec4::new(0.0, 0.1, 0.01, 1.0);

        unsafe {
            gl::BlendColor(0.00, 0.1, 0.01, 1.00);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_mean.len() - 1,
            );
        }
    }

    pub fn set_draw_config(&mut self, draw_config: &DrawConfig) {
        self.draw_config.buffer(draw_config);
    }

    fn generate_vertices_buffer(vertices: &Vec<Vec2>) -> VertexBuffer<Vec2> {
        VertexBuffer::new(Self::VERTICES_ATTRIBUTES.to_vec(), &vertices)
    }

    fn generate_vertices(bar_segments: u32) -> Vec<Vec2> {
        let vertices_count = 2 + bar_segments * 2;
        let mut vertices: Vec<Vec2> = Vec::with_capacity(vertices_count as usize);
        vertices.push(Vec2::new(0.0, 0.0));
        vertices.push(Vec2::new(0.0, 1.0));
        for i in 1..=bar_segments {
            let x = i as f32 / bar_segments as f32;
            vertices.push(Vec2::new(x, 0.0));
            vertices.push(Vec2::new(x, 1.0));
        }
        vertices
    }

    const VERTICES_ATTRIBUTES: [VertexAttribute; 1] = [VertexAttribute::new(
        Self::VERTICES_BINDING_POINT,
        2,
        size_of::<Vec2>() as i32,
        0,
        0,
    )];
    const MAGNITUDE_ATTRIBUTES: [VertexAttribute; 1] = [VertexAttribute::new(
        Self::MAGNITUDES_BINDING_POINT,
        1,
        size_of::<f32>() as i32,
        0,
        1,
    )];
    const VERTEX_SHADER: &'static str = include_str!("../resources/barplot.vert");
    const FRAGMENT_SHADER: &'static str = include_str!("../resources/basic.frag");
    const VERTICES_BINDING_POINT: u32 = 0;
    const MAGNITUDES_BINDING_POINT: u32 = 1;
    const CLIENT_SIZE_BINDING_POINT: u32 = 1;
    const VIEW_MATRIX_BINDING_POINT: u32 = 0;
    const MIN_MAX_BINDING_POINT: u32 = 3;
}

pub struct SpectrumRenderer {
    program: SpectrumRendererShader,
    spectrums: Option<(Spectrum, Spectrum, Spectrum)>,
    // uniforms
    render_config: RenderConfig,
    draw_config: DrawConfig,
}

impl SpectrumRenderer {
    pub fn new(bar_segments: u32) -> Self {
        if bar_segments < 1 {
            panic!("BarPlotConfig.bar_segments must be greater than 0");
        }
        if bar_segments % 2 != 0 {
            panic!("BarPlotConfig.bar_segments must be even");
        }

        Self {
            program: SpectrumRendererShader::new(bar_segments),
            spectrums: None,
            render_config: RenderConfig::new(),
            draw_config: DrawConfig::new(),
        }
    }

    fn exp_decay(start: f32, end: f32, half_life: f32, time_step: f32) -> f32 {
        let decay = (2.0_f32).ln() / half_life;
        end + (start - end) * (-time_step * decay).exp()
    }

    pub fn set_spectrum(&mut self, spectrum: &Spectrum, time_step: Duration) {
        if let Some((last, mean, peek)) = &mut self.spectrums {
            *last = spectrum.clone();
            mean.into_iter()
                .zip(spectrum.into_iter())
                .for_each(|(current, new)| {
                    // *current = *current * 0.9 + new * 0.1;
                    *current = Self::exp_decay(*current, *new, 0.1, time_step.as_secs_f32());
                });
            peek.into_iter()
                .zip(spectrum.into_iter())
                .for_each(|(current, new)| {
                    // *current = *current * 0.9 + new * 0.1;
                    if new > current {
                        *current = *new;
                    } else {
                        // *current = *current * 0.9;
                        *current = Self::exp_decay(*current, 0.0, 0.1, time_step.as_secs_f32());
                    }
                });
        } else {
            self.spectrums = Some((spectrum.clone(), spectrum.clone(), spectrum.clone()));
        }
        let (last, mean, peek) = self.spectrums.as_ref().unwrap();
    }

    pub fn set_render_size(&mut self, size: (u32, u32)) {
        self.render_config.client_size = Vector2::new(size.0 as f32, size.1 as f32);
        let view_matrix =
            Matrix4::<f32>::orthographic_rh(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.render_config.projection = view_matrix;

        self.program.render_config.buffer(&self.render_config);
    }

    pub fn render(&mut self) {
        if self.spectrums.is_none() {
            return;
        }

        let (last, mean, peek) = self.spectrums.as_ref().unwrap();

        self.program.set_magnitdes(mean.as_slice());
        self.program.set_draw_config(&self.draw_config);

        // render mean_spectrum
        self.spectrum_mean.bind(Self::MAGNITUDES_BINDING_POINT);
        self.spectrum_mean.store_array(mean.as_slice());
        let min = mean.into_iter().cloned().reduce(f32::min).unwrap();
        let max = mean.into_iter().cloned().reduce(f32::max).unwrap() * 0.9;
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.min_max_values
            .buffer_subdata(&(mean.len() as u32), size_of::<Vec2>() as isize);
        self.min_max_values
            .buffer_subdata(&(1.0f32), size_of::<Vec3>() as isize);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.00, 0.1, 0.01, 1.00);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_mean.len() - 1,
            );
        }

        // render peek_slow_falling spectrum
        self.spectrum_peek.bind(Self::MAGNITUDES_BINDING_POINT);
        self.spectrum_peek.store_array(peek.as_slice());
        let min = peek.into_iter().cloned().reduce(f32::min).unwrap();
        let max = peek.into_iter().cloned().reduce(f32::max).unwrap() * 0.9;
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.min_max_values
            .buffer_subdata(&(peek.len() as u32), size_of::<Vec2>() as isize);
        self.min_max_values
            .buffer_subdata(&(1.05f32), size_of::<Vec3>() as isize);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.5, 0.005, 0.0, 1.0);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_peek.len() - 1,
            );
        }
        // render current spectrum
        self.spectrum_current.bind(Self::MAGNITUDES_BINDING_POINT);
        self.spectrum_current.store_array(last.as_slice());
        let min = last.into_iter().cloned().reduce(f32::min).unwrap();
        let max = last.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.min_max_values
            .buffer_subdata(&(last.len() as u32), size_of::<Vec2>() as isize);
        self.min_max_values
            .buffer_subdata(&(1.0f32), size_of::<Vec3>() as isize);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_current.len() - 1,
            );
        }

        self.vertex_array.unbind();
        self.shader.disable();
    }

    pub fn render2(&mut self) {
        self.shader.enable();
        self.vertex_array.bind();

        self.client_size.bind(Self::CLIENT_SIZE_BINDING_POINT);
        self.view_matrix.bind(Self::VIEW_MATRIX_BINDING_POINT);

        let (last, mean, peek) = self.spectrums.as_ref().unwrap();

        // render mean_spectrum
        let min = mean.into_iter().cloned().reduce(f32::min).unwrap();
        let max = mean.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.spectrum_mean.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.00, 0.1, 0.01, 1.00);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_mean.len() - 1,
            );
        }

        // render peek_slow_falling spectrum
        let min = peek.into_iter().cloned().reduce(f32::min).unwrap();
        let max = peek.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.spectrum_peek.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.5, 0.005, 0.0, 1.0);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_peek.len() - 1,
            );
        }

        // render current spectrum
        let min = last.into_iter().cloned().reduce(f32::min).unwrap();
        let max = last.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.spectrum_current.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_current.len() - 1,
            );
        }

        self.vertex_array.unbind();
        self.shader.disable();
    }

    fn render_pass(&mut self, spectrum: &Spectrum) {
        let (min, max) = spectrum
            .into_iter()
            .copied()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| {
                (min.min(x), max.max(x))
            });

        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.spectrum_current.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.spectrum_current.len() - 1,
            );
        }
    }
}
