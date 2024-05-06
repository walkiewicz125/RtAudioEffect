use std::{mem::size_of, os::raw::c_void};

use super::vertex_attribute::VertexAttribute;

pub struct VertexBuffer<T> {
    pub buffer_id: u32,
    pub attributes: Vec<VertexAttribute>,
    pub size: usize,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T> VertexBuffer<T> {
    pub fn new(attributes: Vec<VertexAttribute>, data_buffer: &[T]) -> Self {
        let mut buffer_id = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
        };

        let mut return_instance = Self {
            buffer_id,
            attributes,
            size: 0,
            phantom_data: std::marker::PhantomData,
        };

        return_instance.bind();
        return_instance.buffer_data(data_buffer);

        return_instance
    }

    pub fn buffer_data(&mut self, data: &[T]) {
        self.size = data.len();

        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );
        }

        for attribute in self.attributes.iter() {
            attribute.enable();
        }
    }
    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_id) };
    }

    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) };
    }

    pub fn size(&self) -> i32 {
        self.size as i32
    }
}

pub struct VertexBufferGeneric {
    pub buffer_id: u32,
    pub attributes: Vec<VertexAttribute>,
}

impl VertexBufferGeneric {
    pub fn new(attributes: Vec<VertexAttribute>) -> Self {
        let mut buffer_id = 0;

        unsafe {
            gl::GenBuffers(1, &mut buffer_id);
        };

        let return_instance = Self {
            buffer_id,
            attributes,
        };

        return_instance
    }

    pub fn allocate_buffer(&self, size: usize) {
        unsafe {
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size as isize,
                std::ptr::null(),
                gl::STATIC_DRAW,
            );
        }
    }

    pub fn buffer_subdata<T>(&self, offset: isize, data: &[T]) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                offset,
                data.len() as isize * size_of::<T>() as isize,
                data.as_ptr() as *const c_void,
            );
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer_id) };
    }

    #[allow(dead_code)]
    pub fn unbind(&self) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0) };
    }

    pub fn set_attributes(&self) {
        for attribute in self.attributes.iter() {
            attribute.enable();
        }
    }
}
