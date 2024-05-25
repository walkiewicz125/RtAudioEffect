use std::{mem::size_of, ptr};

pub struct UniformBuffer {
    uniform_buffer_id: u32,
}

impl UniformBuffer {
    pub fn new(size: isize) -> Self {
        let mut uniform_buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut uniform_buffer_id);
            gl::BindBuffer(gl::UNIFORM_BUFFER, uniform_buffer_id);
            gl::BufferData(gl::UNIFORM_BUFFER, size, ptr::null(), gl::DYNAMIC_DRAW);
        }

        Self { uniform_buffer_id }
    }

    pub fn new_for<T>() -> Self {
        Self::new(size_of::<T>() as isize)
    }

    pub fn bind(&self, binding_point: u32) {
        unsafe {
            gl::BindBufferBase(gl::UNIFORM_BUFFER, binding_point, self.uniform_buffer_id);
        }
    }

    pub fn buffer_subdata<T>(&mut self, data: &T, offset: isize) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.uniform_buffer_id);
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                offset,
                size_of::<T>() as isize,
                data as *const T as *const _,
            );
            gl::BindBuffer(gl::UNIFORM_BUFFER, 0);
        }
    }

    pub fn allocate(&self, size: isize) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.uniform_buffer_id);
            gl::BufferData(gl::UNIFORM_BUFFER, size, ptr::null(), gl::DYNAMIC_DRAW);
        }
    }
}
