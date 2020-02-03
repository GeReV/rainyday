use crate::render_gl::Texture;

pub struct FrameBuffer {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl FrameBuffer {
    pub fn new(gl: &gl::Gl) -> FrameBuffer {
        let mut fbo: gl::types::GLuint = 0;

        unsafe {
            gl.GenFramebuffers(1, &mut fbo);
        }

        FrameBuffer {
            id: fbo,
            gl: gl.clone(),
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn attach_texture(&self, texture: &Texture) {
        unsafe {
            self.gl.FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.id(),
                0,
            );
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteFramebuffers(1, &self.id);
        }
    }
}
