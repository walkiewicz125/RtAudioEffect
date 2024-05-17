use glam::{Mat4, Vec2};
use glamour::Matrix4;

use crate::ui::plot::details::{
    shader_program::ShaderProgram,
    storage_buffer::{self, StorageBuffer, StorageBufferArray},
    uniform_buffer::UniformBuffer,
    vertex_array::VertexArray,
};

pub struct SpectrumRenderer {
    shader: ShaderProgram,
    vertex_array: VertexArray,
    client_size: UniformBuffer,
    view_matrix: UniformBuffer,
    min_max: UniformBuffer,
    storage_buffer: StorageBufferArray<f32>,
}
impl SpectrumRenderer {
    const VERTEX_SHADER: &'static str = include_str!("../resources/barplot.vert");
    const FRAGMENT_SHADER: &'static str = include_str!("../resources/basic.frag");
    const BAR_VALUES_BUFFER_BINDING_POINT: u32 = 0;
    const CLIENT_SIZE_BINDING_POINT: u32 = 1;
    const VIEW_MATRIX_BINDING_POINT: u32 = 0;
    const MIN_MAX_BINDING_POINT: u32 = 3;
    const VERTICES_COUNT: i32 = 6;

    pub fn new() -> Self {
        let shader = ShaderProgram::new_from_string(Self::VERTEX_SHADER, Self::FRAGMENT_SHADER)
            .expect("Failed to create SpectrumRenderer ShaderProgram");
        let vertex_array = VertexArray::new();
        let client_size = UniformBuffer::new_for::<Vec2>();
        let view_matrix = UniformBuffer::new_for::<Mat4>();
        let min_max = UniformBuffer::new_for::<Vec2>();
        let storage_buffer = StorageBufferArray::new();

        Self {
            shader,
            vertex_array,
            client_size,
            view_matrix,
            min_max,
            storage_buffer,
        }
    }

    pub fn set_spectrum(&mut self, spectrum: &[f32]) {
        let min = spectrum.into_iter().cloned().reduce(f32::min).unwrap();
        let max = spectrum.into_iter().cloned().reduce(f32::max).unwrap();

        self.min_max.buffer_subdata(&Vec2::new(min, max), 0);

        self.storage_buffer.store_array(spectrum);
    }

    pub fn set_render_size(&mut self, size: (u32, u32)) {
        self.client_size
            .buffer_subdata(&Vec2::new(size.0 as f32, size.1 as f32), 0);

        let view_matrix =
            Matrix4::<f32>::orthographic_rh(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.view_matrix.buffer_subdata(&view_matrix, 0);
    }

    pub fn render(&self) {
        self.shader.enable();
        self.vertex_array.bind();

        self.storage_buffer
            .bind(Self::BAR_VALUES_BUFFER_BINDING_POINT);

        self.client_size.bind(Self::CLIENT_SIZE_BINDING_POINT);
        self.view_matrix.bind(Self::VIEW_MATRIX_BINDING_POINT);
        self.min_max.bind(Self::MIN_MAX_BINDING_POINT);

        unsafe {
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                Self::VERTICES_COUNT * self.storage_buffer.len(),
            );
        }

        self.vertex_array.unbind();
        self.shader.disable();
    }
}
