use glam::{Mat4, Vec2, Vec4};

use super::{
    lines_shader::LinesShader, primitives::storage_buffer::StorageBuffer, renderer::Renderer,
};

pub struct LinesRenderer {
    shader: LinesShader,
    storage: StorageBuffer,
}

impl LinesRenderer {
    pub fn new() -> LinesRenderer {
        let shader = LinesShader::new().expect(&format!("Failed to create LinesShader",));

        let storage = StorageBuffer::new();
        LinesRenderer { shader, storage }
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

    pub fn set_line_strip_open(&mut self, lines_points: &[Vec2]) {
        assert!(
            lines_points.len() > 0,
            "Closed line strip need to be at least 1 elements long"
        );

        let mut points: Vec<Vec2> = vec![
            Vec2 {
                ..Default::default()
            };
            0
        ];
        points.push(lines_points[0]);
        points.extend_from_slice(lines_points);
        points.push(lines_points[lines_points.len() - 1]);

        self.storage.store_array(&points);
    }
    pub fn set_line_strip_closed(&mut self, lines_points: &[Vec2]) {
        assert!(
            lines_points.len() > 1,
            "Closed line strip need to be at least 2 elements long"
        );

        let mut points: Vec<Vec2> = vec![
            Vec2 {
                ..Default::default()
            };
            0
        ];
        points.push(lines_points[lines_points.len() - 1]);
        points.extend_from_slice(lines_points);
        points.push(lines_points[0]);
        points.push(lines_points[1]);

        self.storage.store_array(&points);
    }

    pub fn set_line_width(&mut self, line_width: f32) {
        self.shader.set_line_width(line_width);
    }

    pub fn set_line_color(&mut self, line_color: Vec4) {
        self.shader.set_line_color(line_color);
    }
}

impl Renderer for LinesRenderer {
    fn render(&self) {
        self.shader.draw(&self.storage);
    }
}
