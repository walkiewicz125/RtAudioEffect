use glam::Mat4;

use super::{
    barplot_shader::BarplotShader, primitives::storage_buffer::StorageBuffer, renderer::Renderer,
};

pub struct SpectrogramRenderer {
    frame_buffer_name: u32,
    rendered_texture: u32,

    shader: BarplotShader,
    resolution: (u32, u32),
    flip_vertically: bool,
}

impl SpectrogramRenderer {
    pub fn new(resolution: (u32, u32)) -> SpectrogramRenderer {
        let shader = BarplotShader::new().expect(&format!("Failed to create BarplotShader",));

        let mut frame_buffer_name: u32 = 0;
        let mut rendered_texture: u32 = 0;
        let resolution_scaling = 4.0;
        unsafe {
            // The framebuffer, which regroups 0, 1, or more textures, and 0 or 1 depth buffer.
            gl::GenFramebuffers(1, &mut frame_buffer_name);
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer_name);

            // The texture we're going to render to
            gl::GenTextures(1, &mut rendered_texture);

            // "Bind" the newly created texture : all future texture functions will modify this texture
            gl::BindTexture(gl::TEXTURE_2D, rendered_texture);

            // Give an empty image to OpenGL ( the last "0" )
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R32F as i32,
                (resolution.0 as f32) as i32,
                (resolution.1 as f32) as i32,
                0,
                gl::RED,
                gl::FLOAT,
                std::ptr::null(),
            );

            // Poor filtering. Needed !
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);

            // Set "renderedTexture" as our colour attachement #0
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, rendered_texture, 0);

            // Set the list of draw buffers.
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);

            // Always check that our framebuffer is ok
            assert!(
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE,
                "CheckFramebufferStatus is not FRAMEBUFFER_COMPLETE"
            );
        }

        let storage = StorageBuffer::new();
        SpectrogramRenderer {
            frame_buffer_name,
            rendered_texture,
            shader,
            resolution,
            flip_vertically: false,
        }
    }

    pub fn set_view(&mut self, resolution: (u32, u32)) {
        self.resolution = resolution;

        let view_matrix: Mat4;
        if self.flip_vertically {
            view_matrix = Mat4::orthographic_rh(
                0.0,
                self.resolution.0 as f32,
                self.resolution.1 as f32,
                0.0,
                -1.0,
                1.0,
            );
        } else {
            view_matrix = Mat4::orthographic_rh(
                0.0,
                resolution.0 as f32,
                0.0,
                resolution.1 as f32,
                -1.0,
                1.0,
            );
        }
        self.shader.set_projection(view_matrix);
        self.shader.set_client_size(self.resolution);
    }

    pub fn with_view(mut self, client_size: (u32, u32)) -> Self {
        self.set_view(client_size);
        self
    }

    pub fn flip_vertically(&mut self, flip: bool) {
        self.flip_vertically = flip;
        self.set_view(self.resolution);
    }

    pub fn set_texture_data(&mut self, texture_data: &[f32], resolution: (u32, u32)) {
        let mut data = vec![];
        for p in texture_data {
            let v = (*p);
            let b = (2.0 - 0.25 - v).min(1.0).max(0.0);
            let r = (-1.0 + 0.25 + v).min(1.0).max(0.0);
            let g = 0.0;
            data.push(r);
            data.push(g);
            data.push(b);
        }
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_name);

            gl::BindTexture(gl::TEXTURE_2D, self.rendered_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB32F as i32,
                resolution.0 as i32,
                resolution.1 as i32,
                0,
                gl::RGB,
                gl::FLOAT,
                data.as_ptr().cast(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn get_texture_id(&self) -> u32 {
        self.rendered_texture
    }
}
