use std::os::raw::c_void;

pub struct VertexAttribute {
    pub index: u32,
    pub size: i32,
    pub stride: i32,
    pub offset_pointer: *const c_void,
}

pub trait PrimitiveVerticesBuffer {
    fn get_vertex_attribute(&self) -> &[VertexAttribute];
    fn get_vertices_count(&self) -> i32;
    fn get_data(&self) -> *const c_void;
    fn get_length(&self) -> isize;
}
