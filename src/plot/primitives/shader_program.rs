use super::helpers::*;
use std::ptr;

struct ShaderHolder {
    shader_id: u32,
}

enum ShaderType {
    VertexShader,
    FragmentShader,
}

pub struct ShaderProgram {
    program_id: u32,
}

impl ShaderProgram {
    pub fn new_from_string(vertex_shader: &str, fragment_shader: &str) -> Option<ShaderProgram> {
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

            return None;
        };

        Some(ShaderProgram { program_id })
    }

    pub fn enable(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    pub fn disable(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.program_id) };
    }
}

impl ShaderHolder {
    fn new_from_string(shader_source: &str, shader_type: ShaderType) -> Option<ShaderHolder> {
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
                return None;
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
