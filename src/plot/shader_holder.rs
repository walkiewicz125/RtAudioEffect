use super::helpers::*;
use std::{process, ptr};

pub struct ShaderHolder {
    pub shader_id: u32,
}

pub enum ShaderType {
    VertexShader,
    FragmentShader,
}

impl ShaderHolder {
    pub fn new_from_string(shader_source: &str, shader_type: ShaderType) -> Option<ShaderHolder> {
        let shader_source_ptr = shader_source.as_ptr().cast();
        let shader_source_len = shader_source.len() as i32;
        let shader_id;

        unsafe {
            shader_id = match shader_type {
                ShaderType::VertexShader => gl::CreateShader(gl::VERTEX_SHADER),
                ShaderType::FragmentShader => gl::CreateShader(gl::FRAGMENT_SHADER),
            };

            gl::ShaderSource(
                shader_id,
                1,
                ptr::addr_of!(shader_source_ptr),
                ptr::addr_of!(shader_source_len),
            );
            gl::CompileShader(shader_id);

            if let Err(shader_error) = check_shader_compilation_status(shader_id) {
                eprint!("Shader compilation error. Error message: {}", shader_error);
                process::exit(1);
            };
        };

        Some(ShaderHolder { shader_id })
    }
}

impl Drop for ShaderHolder {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.shader_id) };
    }
}
