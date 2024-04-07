use glam::Mat4;
use log::debug;

use super::{
    primitives::storage_buffer::StorageBuffer, renderer::Renderer, MagnitudeTimelineShader,
};

pub struct MagnitudeTimelineRenderer {
    shader: MagnitudeTimelineShader,
    storage: StorageBuffer,
    client_size: (u32, u32),
    flip_vertically: bool,
}

impl MagnitudeTimelineRenderer {
    pub fn new() -> MagnitudeTimelineRenderer {
        let shader = MagnitudeTimelineShader::new()
            .expect(&format!("Failed to create MagnitudeTimelineShader",));

        let storage = StorageBuffer::new();
        MagnitudeTimelineRenderer {
            shader,
            storage,
            client_size: (0, 0),
            flip_vertically: false,
        }
    }

    pub fn with_view(mut self, client_size: (u32, u32)) -> Self {
        self.set_view(client_size);
        self
    }

    pub fn set_view(&mut self, client_size: (u32, u32)) {
        self.client_size = client_size;

        let view_matrix: Mat4;
        if self.flip_vertically {
            view_matrix = Mat4::orthographic_rh(
                0.0,
                self.client_size.0 as f32,
                self.client_size.1 as f32,
                0.0,
                -1.0,
                1.0,
            );
        } else {
            view_matrix = Mat4::orthographic_rh(
                0.0,
                client_size.0 as f32,
                0.0,
                client_size.1 as f32,
                -1.0,
                1.0,
            );
        }
        self.shader.set_projection(view_matrix);
        self.shader.set_client_size(self.client_size);
    }

    pub fn flip_vertically(&mut self, flip: bool) {
        self.flip_vertically = flip;
        self.set_view(self.client_size);
    }

    pub fn set_magnitude_timelie(&mut self, spectrum_data: &Vec<Vec<f32>>) {
        let mut converted_data: Vec<f32> = vec![];
        for data in spectrum_data {
            converted_data.push(data[0]);
        }
        self.storage.store_array(&converted_data);
    }
}

impl Renderer for MagnitudeTimelineRenderer {
    fn render(&self) {
        self.shader.draw(&self.storage);
    }
}
