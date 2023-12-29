use std::{mem::size_of, ptr::null};

pub struct StorageBuffer {
    storage_buffer_id: u32,
    element_count: i32,
}

impl StorageBuffer {
    pub fn new() -> Self {
        let mut buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
        }

        StorageBuffer {
            storage_buffer_id: buffer_id,
            element_count: 0,
        }
    }

    pub fn bind(&self, binding_point: u32) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);
            gl::BindBufferBase(
                gl::SHADER_STORAGE_BUFFER,
                binding_point,
                self.storage_buffer_id,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    pub fn store_array<T>(&mut self, data: &[T]) {
        self.element_count = data.len() as i32;
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);

            // Allocate buffer
            let len_size = size_of::<u32>() as isize;
            let array_size = (data.len() * size_of::<T>()) as isize;
            let total_size = len_size + array_size;
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                total_size,
                null(),
                gl::DYNAMIC_DRAW,
            );

            // Store number of elements
            let len = data.len() as u32;
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                len_size,
                &len as *const u32 as *const _,
            );

            // Store array
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                len_size,
                array_size,
                data.as_ptr().cast(),
            );

            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    pub fn len(&self) -> i32 {
        self.element_count
    }
}
