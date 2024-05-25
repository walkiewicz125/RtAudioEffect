pub struct VertexArray {
    pub vertex_array_id: u32,
}

impl VertexArray {
    pub fn new() -> Self {
        let mut vertex_array_id = 0;
        unsafe { gl::GenVertexArrays(1, &mut vertex_array_id) };

        Self { vertex_array_id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.vertex_array_id);
        }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindVertexArray(0) };
    }
}
