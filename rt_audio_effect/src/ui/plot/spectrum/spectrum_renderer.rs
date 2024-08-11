use std::{mem::size_of, time::Duration};

use glam::{Mat4, Vec2, Vec3, Vec4};
use glamour::Matrix4;

use crate::{
    audio_analyzer::Spectrum,
    ui::plot::details::{
        shader_program::ShaderProgram, storage_buffer::StorageBufferArray,
        uniform_buffer::UniformBuffer, vertex_array::VertexArray,
        vertex_attribute::VertexAttribute, vertex_buffer::VertexBuffer,
    },
};

pub struct SpectrumRenderer {
    vertices: Vec<Vec2>,
    bar_shaper: Vec<f32>,
    spectrums: Option<(Spectrum, Spectrum, Spectrum)>,
    // shader
    shader: ShaderProgram,
    // vertex array
    vertex_array: VertexArray,
    vertices_buffer: VertexBuffer<Vec2>,
    // uniforms
    min_max_values: UniformBuffer,
    client_size: UniformBuffer,
    view_matrix: UniformBuffer,
    // buffers
    magnitudes: StorageBufferArray<f32>,
    magnitudes2: StorageBufferArray<f32>,
    magnitudes3: StorageBufferArray<f32>,
    shaper: StorageBufferArray<f32>,
}

impl SpectrumRenderer {
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
        let min_max_values = UniformBuffer::new_for::<Vec4>();
        let client_size = UniformBuffer::new_for::<Vec2>();
        let view_matrix = UniformBuffer::new_for::<Mat4>();

        let magnitudes = StorageBufferArray::new();
        let magnitudes2 = StorageBufferArray::new();
        let magnitudes3 = StorageBufferArray::new();
        let mut shaper = StorageBufferArray::new();
        let bar_shaper = Self::generate_shaper(bar_segments);
        vertex_array.bind();
        shaper.store_array(bar_shaper.as_slice());

        let vertices = Self::generate_vertices(bar_segments);
        let vertices_buffer = Self::generate_vertices_buffer(&vertices);
        Self {
            vertices,
            bar_shaper,
            spectrums: None,
            shader,
            vertex_array,
            vertices_buffer,
            min_max_values,
            client_size,
            view_matrix,
            magnitudes,
            magnitudes2,
            magnitudes3,
            shaper,
        }
    }

    fn generate_vertices_buffer(vertices: &Vec<Vec2>) -> VertexBuffer<Vec2> {
        VertexBuffer::new(Self::VERTICES_ATTRIBUTES.to_vec(), &vertices)
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
        self.magnitudes.store_array(last.as_slice());
        self.magnitudes2.store_array(mean.as_slice());
        self.magnitudes3.store_array(peek.as_slice());
    }

    pub fn set_render_size(&mut self, size: (u32, u32)) {
        self.client_size
            .buffer_subdata(&Vec2::new(size.0 as f32, size.1 as f32), 0);

        let view_matrix =
            Matrix4::<f32>::orthographic_rh(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.view_matrix.buffer_subdata(&view_matrix, 0);
    }

    pub fn render(&mut self) {
        if self.spectrums.is_none() {
            return;
        }

        self.shader.enable();
        self.vertex_array.bind();
        self.client_size.bind(Self::CLIENT_SIZE_BINDING_POINT);
        self.view_matrix.bind(Self::VIEW_MATRIX_BINDING_POINT);
        self.shaper.bind(Self::SHAPER_BUFFER_BINDING_POINT);

        self.vertices_buffer.bind();
        self.vertices_buffer.buffer_data(self.vertices.as_slice());

        let (last, mean, peek) = self.spectrums.as_ref().unwrap();

        // render mean_spectrum
        self.magnitudes2.bind(Self::MAGNITUDES_BINDING_POINT);
        self.magnitudes2.store_array(mean.as_slice());
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
                self.magnitudes2.len() - 1,
            );
        }

        // render peek_slow_falling spectrum
        self.magnitudes3.bind(Self::MAGNITUDES_BINDING_POINT);
        self.magnitudes3.store_array(peek.as_slice());
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
                self.magnitudes3.len() - 1,
            );
        }
        // render current spectrum
        self.magnitudes.bind(Self::MAGNITUDES_BINDING_POINT);
        self.magnitudes.store_array(last.as_slice());
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
                self.magnitudes.len() - 1,
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
        self.magnitudes2.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.00, 0.1, 0.01, 1.00);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.magnitudes2.len() - 1,
            );
        }

        // render peek_slow_falling spectrum
        let min = peek.into_iter().cloned().reduce(f32::min).unwrap();
        let max = peek.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.magnitudes3.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.5, 0.005, 0.0, 1.0);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.magnitudes3.len() - 1,
            );
        }

        // render current spectrum
        let min = last.into_iter().cloned().reduce(f32::min).unwrap();
        let max = last.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max_values.buffer_subdata(&Vec2::new(min, max), 0);
        self.magnitudes.bind(Self::MAGNITUDES_BINDING_POINT);
        self.min_max_values.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices.len() as i32,
                self.magnitudes.len() - 1,
            );
        }

        self.vertex_array.unbind();
        self.shader.disable();
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

    fn generate_shaper(bar_segments: u32) -> Vec<f32> {
        let count = 1 + 2 * bar_segments as usize;
        println!("{:?}", count);
        let shaper = apodize::hanning_iter(count)
            .map(|v| v as f32)
            .take((count + 1) / 2)
            .inspect(|v| println!("{:?}", v))
            .collect::<Vec<f32>>();

        println!("{:?}", shaper);
        shaper
    }

    const VERTEX_SHADER: &'static str = include_str!("../resources/barplot.vert");
    const FRAGMENT_SHADER: &'static str = include_str!("../resources/basic.frag");
    const VERTICES_BINDING_POINT: u32 = 0;
    const MAGNITUDES_BINDING_POINT: u32 = 1;
    const SHAPER_BUFFER_BINDING_POINT: u32 = 2;
    const CLIENT_SIZE_BINDING_POINT: u32 = 1;
    const VIEW_MATRIX_BINDING_POINT: u32 = 0;
    const MIN_MAX_BINDING_POINT: u32 = 3;
}
