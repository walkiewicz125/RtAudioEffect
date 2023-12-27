use glam::Mat4;
use std::{mem::size_of, process};

use super::helpers::*;
use super::primitive_base::{InstanceBuffer, PrimitiveVerticesBuffer};
use super::shader_holder::*;

pub struct WaveplotRenderer {
    // openGl objects ids:
    program_shader_id: u32,
    vertex_array_id: u32,
    vertices_buffer_id: u32,
    instance_buffer_id: u32,
    view_uniform_id: u32,

    // application internal
    instance_count: i32,
    vertices_count: i32,
}

impl WaveplotRenderer {
    pub fn new_from_string(vertex_shader: &str, fragment_shader: &str) -> Option<WaveplotRenderer> {
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
        let mut instance_buffer_id = 0;
        let mut view_uniform_id = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::GenBuffers(1, &mut vertices_buffer_id);
            gl::GenBuffers(1, &mut instance_buffer_id);
            gl::GenBuffers(1, &mut view_uniform_id);
        }

        Some(WaveplotRenderer {
            program_shader_id: program_id,
            vertex_array_id,
            vertices_buffer_id,
            instance_buffer_id,
            view_uniform_id,
            instance_count: 0,
            vertices_count: 0,
        })
    }

    pub fn set_view_uniform(&self, view_matrix: Mat4) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.view_uniform_id);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Mat4>() as isize,
                view_matrix.to_cols_array().as_ptr().cast(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 1, self.view_uniform_id);
        }
    }

    pub fn buffer_primitive_vertices(&mut self, primitive_vertices: &dyn PrimitiveVerticesBuffer) {
        self.vertices_count = primitive_vertices.get_vertices_count();

        for attribute in primitive_vertices.get_vertex_attribute() {
            unsafe {
                gl::BindVertexArray(self.vertex_array_id);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vertices_buffer_id);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    primitive_vertices.get_length(),
                    primitive_vertices.get_data(),
                    gl::STATIC_DRAW,
                );

                gl::EnableVertexAttribArray(attribute.index);
                gl::VertexAttribPointer(
                    attribute.index,
                    attribute.size,
                    gl::FLOAT,
                    gl::FALSE,
                    attribute.stride,
                    attribute.offset_pointer,
                );
                gl::BindVertexArray(0);
            }
        }
    }

    pub fn buffer_instances(&mut self, instance_buffer: &dyn InstanceBuffer) {
        self.instance_count = instance_buffer.get_instance_count();

        let data = instance_buffer.get_data();
        let data_length = instance_buffer.get_length();

        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, data_length, data, gl::DYNAMIC_DRAW);
        }

        for vertex_attribute in instance_buffer.get_vertex_attributes() {
            unsafe {
                gl::EnableVertexAttribArray(vertex_attribute.index);
                gl::VertexAttribPointer(
                    vertex_attribute.index,
                    vertex_attribute.size,
                    gl::FLOAT,
                    gl::FALSE,
                    vertex_attribute.stride,
                    vertex_attribute.offset_pointer,
                );
                gl::VertexAttribDivisor(vertex_attribute.index, 1);
            }
        }

        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn draw_instances(&self) {
        unsafe {
            gl::UseProgram(self.program_shader_id);
            gl::BindVertexArray(self.vertex_array_id);

            gl::DrawArraysInstanced(gl::TRIANGLES, 0, self.vertices_count, self.instance_count);
            gl::UseProgram(0);
        }
    }
}

impl Drop for WaveplotRenderer {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program_shader_id) };
    }
}
