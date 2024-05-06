use std::{
    mem::size_of,
    ptr::{self, null},
};

pub struct StorageBufferAny {
    storage_buffer_id: u32,
}

impl StorageBufferAny {
    pub fn new() -> Self {
        let mut storage_buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut storage_buffer_id);
        }

        Self { storage_buffer_id }
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

    pub fn buffer_subdata<T>(&mut self, data: &T, offset: isize) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                offset,
                size_of::<T>() as isize,
                data as *const T as *const _,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    pub fn buffer_subdata_array<T>(&mut self, data: &[T], offset: isize) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                offset,
                (data.len() * size_of::<T>()) as isize,
                data.as_ptr().cast(),
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
    pub fn allocate(&self, size: isize) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }
}

pub struct StorageBuffer<T> {
    storage_buffer_id: u32,
    phantom_data: std::marker::PhantomData<T>,
}

pub struct StorageBufferArray<T> {
    storage_buffer_id: u32,
    element_count: i32,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T> StorageBuffer<T> {
    pub fn new() -> Self {
        let mut storage_buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut storage_buffer_id);
        }

        Self {
            storage_buffer_id,
            phantom_data: std::marker::PhantomData,
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

    pub fn store_data(&self, data: &T) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size_of::<T>() as isize,
                data as *const T as *const _,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
}

impl<T> StorageBufferArray<T> {
    pub fn new() -> Self {
        let mut storage_buffer_id = 0;
        unsafe {
            gl::GenBuffers(1, &mut storage_buffer_id);
        }

        Self {
            storage_buffer_id,
            element_count: 0,
            phantom_data: std::marker::PhantomData,
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

    pub fn store_array(&mut self, data: &[T]) {
        self.element_count = data.len() as i32;
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.storage_buffer_id);

            // Allocate buffer
            let len_size = size_of::<T>() as isize;
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
