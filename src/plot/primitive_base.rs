use std::os::raw::c_void;

struct PrimitiveBaseInstanceBuffer {
    instances_buffer_id: u32,
    count: i32,
}

impl PrimitiveBaseInstanceBuffer {
    fn new() -> PrimitiveBaseInstanceBuffer {
        let mut instances_buffer_id: u32 = 0;
        unsafe {
            gl::GenBuffers(1, &mut instances_buffer_id);
        }

        PrimitiveBaseInstanceBuffer {
            instances_buffer_id,
            count: 0,
        }
    }

    fn count(&self) -> i32 {
        self.count
    }

    fn update(&mut self, instance_buffer: &dyn InstanceBuffer) {
        self.count = instance_buffer.get_instance_count();
        let data = instance_buffer.get_data();
        let data_length = instance_buffer.get_length();
        let attributes = instance_buffer.get_vertex_attributes();
        unsafe {
            // vertex
            gl::BindBuffer(gl::ARRAY_BUFFER, self.instances_buffer_id);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                data_length as isize,
                data,
                gl::DYNAMIC_DRAW,
            );

            for attribute in attributes {
                gl::EnableVertexAttribArray(attribute.index);
                gl::VertexAttribPointer(
                    attribute.index,
                    attribute.size,
                    gl::FLOAT,
                    gl::FALSE,
                    attribute.stride,
                    attribute.offset_pointer,
                );
                gl::VertexAttribDivisor(attribute.index, 1);
            }
        }
    }
}

pub struct VertexAttribute {
    pub index: u32,
    pub size: i32,
    pub stride: i32,
    pub offset_pointer: *const c_void,
}

pub trait InstanceBuffer {
    fn get_vertex_attributes(&self) -> &[VertexAttribute];
    fn get_data(&self) -> *const c_void;
    fn get_length(&self) -> isize;
    fn get_instance_count(&self) -> i32;
}

pub trait PrimitiveVerticesBuffer {
    fn get_vertex_attribute(&self) -> &[VertexAttribute];
    fn get_vertices_count(&self) -> i32;
    fn get_data(&self) -> *const c_void;
    fn get_length(&self) -> isize;
}
