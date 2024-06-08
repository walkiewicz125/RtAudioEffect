use std::time::Duration;

use glam::{Mat4, Vec2};
use glamour::Matrix4;

use crate::{
    audio_analyzer::Spectrum,
    ui::plot::details::{
        shader_program::ShaderProgram,
        storage_buffer::{self, StorageBuffer, StorageBufferArray},
        uniform_buffer::UniformBuffer,
        vertex_array::VertexArray,
    },
};

pub struct SpectrumRenderer {
    shader: ShaderProgram,
    vertex_array: VertexArray,
    client_size: UniformBuffer,
    view_matrix: UniformBuffer,
    min_max: UniformBuffer,
    storage_buffer: StorageBufferArray<f32>,
    storage_buffer2: StorageBufferArray<f32>,
    storage_buffer3: StorageBufferArray<f32>,
    spectrums: Option<(Spectrum, Spectrum, Spectrum)>,
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
        let storage_buffer2 = StorageBufferArray::new();
        let storage_buffer3 = StorageBufferArray::new();

        Self {
            shader,
            vertex_array,
            client_size,
            view_matrix,
            min_max,
            storage_buffer,
            storage_buffer2,
            storage_buffer3,
            spectrums: None,
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
        self.storage_buffer.store_array(last.as_slice());
        self.storage_buffer2.store_array(mean.as_slice());
        self.storage_buffer3.store_array(peek.as_slice());
    }

    pub fn set_render_size(&mut self, size: (u32, u32)) {
        self.client_size
            .buffer_subdata(&Vec2::new(size.0 as f32, size.1 as f32), 0);

        let view_matrix =
            Matrix4::<f32>::orthographic_rh(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.view_matrix.buffer_subdata(&view_matrix, 0);
    }

    pub fn render(&mut self) {
        self.shader.enable();
        self.vertex_array.bind();

        self.client_size.bind(Self::CLIENT_SIZE_BINDING_POINT);
        self.view_matrix.bind(Self::VIEW_MATRIX_BINDING_POINT);

        if self.spectrums.is_none() {
            return;
        }

        let (last, mean, peek) = self.spectrums.as_ref().unwrap();

        // render mean_spectrum
        let min = mean.into_iter().cloned().reduce(f32::min).unwrap();
        let max = mean.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max.buffer_subdata(&Vec2::new(min, max), 0);
        self.storage_buffer2
            .bind(Self::BAR_VALUES_BUFFER_BINDING_POINT);
        self.min_max.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.00, 0.1, 0.01, 1.00);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                Self::VERTICES_COUNT * self.storage_buffer2.len(),
            );
        }

        // render peek_slow_falling spectrum
        let min = peek.into_iter().cloned().reduce(f32::min).unwrap();
        let max = peek.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max.buffer_subdata(&Vec2::new(min, max), 0);
        self.storage_buffer3
            .bind(Self::BAR_VALUES_BUFFER_BINDING_POINT);
        self.min_max.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendColor(0.5, 0.005, 0.0, 1.0);
            gl::BlendFunc(gl::CONSTANT_COLOR, gl::DST_ALPHA);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                Self::VERTICES_COUNT * self.storage_buffer3.len(),
            );
        }

        // render current spectrum
        let min = last.into_iter().cloned().reduce(f32::min).unwrap();
        let max = last.into_iter().cloned().reduce(f32::max).unwrap();
        self.min_max.buffer_subdata(&Vec2::new(min, max), 0);
        self.storage_buffer
            .bind(Self::BAR_VALUES_BUFFER_BINDING_POINT);
        self.min_max.bind(Self::MIN_MAX_BINDING_POINT);
        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
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
