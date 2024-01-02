use glam::Mat4;

use super::{barplot_shader::BarplotShader, primitives::storage_buffer::StorageBuffer};

pub struct BarSpectrumRenderer {
    shader: BarplotShader,
    storage: StorageBuffer,
}

impl BarSpectrumRenderer {
    pub fn new() -> BarSpectrumRenderer {
        let shader = BarplotShader::new().expect(&format!("Failed to create BarplotShader",));

        let storage = StorageBuffer::new();
        BarSpectrumRenderer { shader, storage }
    }

    pub fn with_view(self, client_size: (u32, u32)) -> Self {
        self.set_view(client_size);
        self
    }

    pub fn set_view(&self, client_size: (u32, u32)) {
        let view_matrix = Mat4::orthographic_rh(
            0.0,
            client_size.0 as f32,
            0.0,
            client_size.1 as f32,
            -1.0,
            1.0,
        );

        self.shader.set_projection(view_matrix);
        self.shader.set_client_size(client_size);
    }

    pub fn set_spectrum(&mut self, spectrum_data: &[f32]) {
        self.storage.store_array(spectrum_data);
    }

    pub fn render(&self) {
        self.shader.draw(&self.storage);
    }
}