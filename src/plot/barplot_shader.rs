use glam::{Mat4, Vec2};
use std::{mem::size_of, process};

use super::helpers::*;
use super::primitive_base::PrimitiveVerticesBuffer;
use super::shader_holder::*;
use super::storage_buffer::StorageBuffer;

pub struct BarplotShader {
    // openGl objects ids:
    program_shader_id: u32,
    vertex_array_id: u32,
    vertices_buffer_id: u32,
    view_uniform_id: u32,
    view_uniform_id_2: u32,

    // application internal
    vertices_count: i32,
}

impl BarplotShader {
    pub fn new_from_string(vertex_shader: &str, fragment_shader: &str) -> Option<BarplotShader> {
        let vertex = ShaderHolder::new_from_string(vertex_shader, ShaderType::VertexShader);
        let fragment = ShaderHolder::new_from_string(fragment_shader, ShaderType::FragmentShader);

        if vertex.is_none() && fragment.is_none() {
            return None;
        }

        let vertex = vertex.unwrap();
        let fragment = fragment.unwrap();

        let mut program_id: u32 = 0;

        unsafe {
            program_id = gl::CreateProgram();
            gl::AttachShader(program_id, vertex.shader_id);
            gl::AttachShader(program_id, fragment.shader_id);
            gl::LinkProgram(program_id);
        }

        if let Err(shader_error) = check_shader_linking_status(program_id) {
            eprint!("Shader linkage error. Error message: {}", shader_error);
            process::exit(1);
        };

        let mut vertex_array_id = 0;
        let mut vertices_buffer_id = 0;
        let mut view_uniform_id = 0;
        let mut view_uniform_id_2 = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::GenBuffers(1, &mut vertices_buffer_id);
            gl::GenBuffers(1, &mut view_uniform_id);
            gl::GenBuffers(1, &mut view_uniform_id_2);
        }

        Some(BarplotShader {
            program_shader_id: program_id,
            vertex_array_id,
            vertices_buffer_id,
            view_uniform_id,
            view_uniform_id_2,
            vertices_count: 6,
        })
    }

    pub fn set_projection(&self, view_matrix: Mat4) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.view_uniform_id);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Mat4>() as isize,
                view_matrix.to_cols_array().as_ptr().cast(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.view_uniform_id);
        }
    }

    pub fn draw(&self, storage_buffer: &StorageBuffer) {
        unsafe {
            gl::UseProgram(self.program_shader_id);
            gl::BindVertexArray(self.vertex_array_id);
            storage_buffer.bind(0);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices_count * storage_buffer.len());
            gl::UseProgram(0);
        }
    }

    pub(crate) fn set_client_size(&self, client_size: (u32, u32)) {
        let vec = Vec2 {
            x: client_size.0 as f32,
            y: client_size.1 as f32,
        };
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.view_uniform_id_2);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Vec2>() as isize,
                &vec as *const Vec2 as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 1, self.view_uniform_id_2);
        }
    }
}

impl Drop for BarplotShader {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program_shader_id) };
    }
}
