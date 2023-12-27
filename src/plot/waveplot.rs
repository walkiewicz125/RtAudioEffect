use std::{ffi::c_void, mem::size_of};

use glam::Vec2;

use super::{
    primitive_base::{self, InstanceBuffer, VertexAttribute},
    waveplot_renderer::WaveplotRenderer,
};

pub struct WaveplotInstancBuffer {
    instances: Vec<Vec2>,
}

impl WaveplotInstancBuffer {
    pub fn update(&mut self, new_instances: Vec<Vec2>) {
        self.instances = new_instances;
    }

    pub fn default() -> WaveplotInstancBuffer {
        WaveplotInstancBuffer { instances: vec![] }
    }

    pub(crate) fn render(&self, waveplot_renderer: &mut WaveplotRenderer) {
        waveplot_renderer.buffer_instances(self);
        waveplot_renderer.draw_instances();
    }
}

impl WaveplotInstancBuffer {
    const STRIDE: i32 = size_of::<Vec2>() as i32;
    const ATTRIB_OFFSET_BAR_POS_SIZE: usize = 0;
    const WAVEPLOT_SHADER_ATTRIBUTES: [VertexAttribute; 1] = [VertexAttribute {
        index: 2,
        size: 2,
        stride: WaveplotInstancBuffer::STRIDE,
        offset_pointer: WaveplotInstancBuffer::ATTRIB_OFFSET_BAR_POS_SIZE as *const _,
    }];

    pub(crate) fn new() -> WaveplotInstancBuffer {
        WaveplotInstancBuffer::default()
    }
}

impl InstanceBuffer for WaveplotInstancBuffer {
    fn get_vertex_attributes(&self) -> &[VertexAttribute] {
        WaveplotInstancBuffer::WAVEPLOT_SHADER_ATTRIBUTES.as_slice()
    }

    fn get_data(&self) -> *const c_void {
        self.instances.as_ptr().cast()
    }

    fn get_length(&self) -> isize {
        self.instances.len() as isize * size_of::<Vec2>() as isize
    }

    fn get_instance_count(&self) -> i32 {
        self.instances.len() as i32
    }
}
