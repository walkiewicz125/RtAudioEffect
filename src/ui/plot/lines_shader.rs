use glam::{Mat4, Vec2, Vec3, Vec4};
use std::{default, mem::size_of};

use super::primitives::{shader_program::ShaderProgram, storage_buffer::StorageBuffer};

pub struct LinesShader {
    // openGl objects ids:
    shader_program: ShaderProgram,
    vertex_array_id: u32,
    projection_id: u32,
    client_size_id: u32,
    line_width_id: u32,

    // application internal
    vertices_count: i32,
    line_parameters: LineParametersUniform,
}

#[repr(C)]
#[derive(Default)]
struct LineParametersUniform {
    line_width: f32,
    padding1: f32,
    padding2: f32,
    padding3: f32,
    line_color: Vec4,
}

impl LinesShader {
    pub fn new() -> Option<LinesShader> {
        let shader_program = ShaderProgram::new_from_string(
            include_str!("resources/lines.vert"),
            include_str!("resources/basic.frag"),
        )
        .expect(&format!(
            "Failed to create LinesShader: {}, {}.",
            "resources/lines.vert", "resources/basic.frag"
        ));

        let mut vertex_array_id = 0;
        let mut projection_id = 0;
        let mut client_size_id = 0;
        let mut line_width_id = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::GenBuffers(1, &mut projection_id);
            gl::GenBuffers(1, &mut client_size_id);
            gl::GenBuffers(1, &mut line_width_id);
        }

        Some(LinesShader {
            shader_program,
            vertex_array_id,
            projection_id,
            client_size_id,
            line_width_id,
            vertices_count: 12,
            line_parameters: LineParametersUniform::default(),
        })
    }

    pub fn set_projection(&self, view_matrix: Mat4) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.projection_id);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Mat4>() as isize,
                view_matrix.to_cols_array().as_ptr().cast(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 0, self.projection_id);
        }
    }

    pub fn set_client_size(&self, client_size: (u32, u32)) {
        let vec = Vec2 {
            x: client_size.0 as f32,
            y: client_size.1 as f32,
        };
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.client_size_id);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<Vec2>() as isize,
                &vec as *const Vec2 as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 1, self.client_size_id);
        }
    }

    pub fn set_line_width(&mut self, line_width: f32) {
        self.line_parameters.line_width = line_width;
        self.buffer_uniform();
    }

    pub fn set_line_color(&mut self, line_color: Vec4) {
        self.line_parameters.line_color = line_color;
        self.buffer_uniform();
    }

    fn buffer_uniform(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.line_width_id);
            gl::BufferData(
                gl::UNIFORM_BUFFER,
                size_of::<LineParametersUniform>() as isize,
                &self.line_parameters as *const LineParametersUniform as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBufferBase(gl::UNIFORM_BUFFER, 3, self.line_width_id);
        }
    }

    pub fn draw(&self, storage_buffer: &StorageBuffer) {
        self.shader_program.enable();
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
            storage_buffer.bind(0);
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                self.vertices_count * (storage_buffer.len() - 1),
            );
        }
        self.shader_program.disable();
    }
}
