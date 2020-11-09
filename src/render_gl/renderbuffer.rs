pub struct RenderBuffer {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl RenderBuffer {
    pub fn new(gl: &gl::Gl) -> RenderBuffer {
        let mut rbo: gl::types::GLuint = 0;

        unsafe {
            gl.GenRenderbuffers(1, &mut rbo);
        }

        RenderBuffer {
            id: rbo,
            gl: gl.clone(),
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindRenderbuffer(gl::RENDERBUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindRenderbuffer(gl::RENDERBUFFER, 0);
        }
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteRenderbuffers(1, &self.id);
        }
    }
}
