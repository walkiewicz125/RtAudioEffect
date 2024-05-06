use std::os::raw::c_void;

pub struct VertexAttribute {
    pub index: u32,
    pub size: i32,
    pub stride: i32,
    pub offset_pointer: usize,
    pub divisor: u32,
}

impl VertexAttribute {
    pub const fn new(
        index: u32,
        size: i32,
        stride: i32,
        offset_pointer: usize,
        divisor: u32,
    ) -> Self {
        Self {
            index,
            size,
            stride,
            offset_pointer,
            divisor,
        }
    }

    pub fn enable(&self) {
        unsafe {
            gl::EnableVertexAttribArray(self.index);
            gl::VertexAttribPointer(
                self.index,
                self.size,
                gl::FLOAT,
                gl::FALSE,
                self.stride,
                self.offset_pointer as *const c_void,
            );
            gl::VertexAttribDivisor(self.index, self.divisor);
        }
    }
}

impl Clone for VertexAttribute {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            size: self.size,
            stride: self.stride,
            offset_pointer: self.offset_pointer,
            divisor: self.divisor,
        }
    }
}
