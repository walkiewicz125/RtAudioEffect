use std::mem::size_of;

pub struct ElementBuffer {
    buffer_id: u32,
}

impl ElementBuffer {
    pub fn new(indices: &[u32]) -> Self {
        let mut buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, buffer_id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size_of::<u32>() as isize * indices.len() as isize,
                indices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
        }

        Self { buffer_id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
}
