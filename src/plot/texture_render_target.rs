use super::renderer::Renderer;

pub struct TextureRenderTarget {
    frame_buffer_name: u32,
    rendered_texture: u32,
}

impl TextureRenderTarget {
    pub fn new() -> Self {
        let mut frame_buffer_name: u32 = 0;
        let mut rendered_texture: u32 = 0;

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
                gl::RGB as i32,
                10240,
                7680,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
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

        Self {
            frame_buffer_name,
            rendered_texture,
        }
    }

    pub fn render(&self, renderer: &dyn Renderer) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.frame_buffer_name);
            gl::ClearColor(0.455, 0.302, 0.663, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Viewport(0, 0, 10240, 7680); // Render on the whole framebuffer, complete from the lower left corner to the upper right
        }

        renderer.render();

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
}
