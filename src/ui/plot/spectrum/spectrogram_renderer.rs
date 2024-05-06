use glam::{Mat4, Vec2};
use glamour::{Matrix4, Vector2};

use std::mem::size_of;

use crate::{
    audio_analyzer::{Magnitude, Spectrum, TimeSeries},
    ui::plot::details::{
        shader_program::ShaderProgram, storage_buffer::StorageBufferAny,
        uniform_buffer::UniformBuffer, vertex_array::VertexArray,
        vertex_attribute::VertexAttribute, vertex_buffer::VertexBuffer,
    },
};

struct SpectrogramData {
    spectrogram_data: StorageBufferAny,
    width: u32,
    length: u32,
}

impl SpectrogramData {
    pub fn new() -> Self {
        let spectrogram_data = StorageBufferAny::new();
        Self {
            spectrogram_data,
            width: 0,
            length: 0,
        }
    }

    pub fn buffer_data(&mut self, spectrogram: (TimeSeries<Magnitude>, (u32, u32))) {
        self.width = spectrogram.1 .0;
        self.length = spectrogram.1 .1;

        let width_offset = 0;
        let length_offset = size_of::<u32>() as isize;
        let magnitude_buffer_offset = 2 * size_of::<u32>() as isize;
        let (time_series, (width, length)) = spectrogram;

        let mut total_size = size_of::<u32>() as isize * 2; // width + length
        total_size += time_series.get_total_len() as isize * size_of::<f32>() as isize;
        self.spectrogram_data.allocate(total_size);
        self.spectrogram_data.buffer_subdata(&width, width_offset);
        self.spectrogram_data.buffer_subdata(&length, length_offset);

        self.spectrogram_data
            .buffer_subdata_array(time_series.get_data(), magnitude_buffer_offset);
    }

    fn bind(&self, binding_point: u32) {
        self.spectrogram_data.bind(binding_point);
    }

    fn size(&self) -> i32 {
        self.length as i32 * self.width as i32
    }
}

pub struct SpectrogramRenderer {
    shader_program: ShaderProgram,
    vertex_array: VertexArray,
    vertices_buffer: VertexBuffer<Vector2>,
    client_size: UniformBuffer,
    view_matrix: UniformBuffer,
    spectrogram_data: SpectrogramData,
}

impl SpectrogramRenderer {
    const SPECTROGRAM_DATA_BINDING_POINT: u32 = 2;
    const CLIENT_SIZE_BINDING_POINT: u32 = 1;
    const VIEW_MATRIX_BINDING_POINT: u32 = 0;

    pub fn new() -> Self {
        let shader_program =
            ShaderProgram::new_from_string(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)
                .expect("Failed to create RectangleShader");

        shader_program.enable();
        let vertex_array = VertexArray::new();
        vertex_array.bind();
        let vertices_buffer = Self::generate_vertices_buffer();
        vertex_array.unbind();
        shader_program.disable();
        let client_size = UniformBuffer::new_for::<Vec2>();
        let view_matrix = UniformBuffer::new_for::<Mat4>();
        let spectrogram_data = SpectrogramData::new();

        Self {
            vertex_array,
            shader_program,
            vertices_buffer,
            client_size,
            view_matrix,
            spectrogram_data,
        }
    }

    pub fn render(&self) {
        self.shader_program.enable();
        self.vertex_array.bind();

        self.client_size.bind(Self::CLIENT_SIZE_BINDING_POINT);
        self.view_matrix.bind(Self::VIEW_MATRIX_BINDING_POINT);
        self.spectrogram_data
            .bind(Self::SPECTROGRAM_DATA_BINDING_POINT);

        unsafe {
            gl::DrawArraysInstanced(
                gl::TRIANGLE_STRIP,
                0,
                self.vertices_buffer.size(),
                self.spectrogram_data.size(),
            );
        }

        self.vertex_array.unbind();
        self.shader_program.disable();
    }

    pub fn set_render_size(&mut self, size: (u32, u32)) {
        self.client_size
            .buffer_subdata(&Vec2::new(size.0 as f32, size.1 as f32), 0);

        let view_matrix =
            Matrix4::<f32>::orthographic_rh(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.view_matrix.buffer_subdata(&view_matrix, 0);
    }

    pub fn buffer_data(&mut self, spectrogram: (TimeSeries<Magnitude>, (u32, u32))) {
        self.spectrogram_data.buffer_data(spectrogram);
    }

    fn generate_vertices_buffer() -> VertexBuffer<Vector2> {
        VertexBuffer::new(Self::VERTICES_ATTRIBUTES.to_vec(), &Self::QUAD.to_vec())
    }
}

impl SpectrogramRenderer {
    const FRAGMENT_SHADER: &'static str = include_str!("../resources/spectrogram.frag");
    const VERTEX_SHADER: &'static str = include_str!("../resources/spectrogram.vert");

    const VERTICES_ATTRIBUTES: [VertexAttribute; 1] = [VertexAttribute::new(
        0,
        2,
        size_of::<Vector2>() as i32,
        0,
        0,
    )];

    const VERTICES: usize = 6;
    const QUAD: [Vector2; Self::VERTICES] = [
        // quad
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
    ];
}
