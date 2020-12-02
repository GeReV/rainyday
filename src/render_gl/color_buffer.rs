use nalgebra as na;

pub struct ColorBuffer {
    pub color: na::Vector4<f32>,
}

impl ColorBuffer {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> ColorBuffer {
        ColorBuffer {
            color: na::Vector4::new(r, g, b, a),
        }
    }

    pub fn from_rgb(r: f32, g: f32, b: f32) -> ColorBuffer {
        Self::from_rgba(r, g, b, 1.0)
    }

    pub fn update_color(&mut self, color: na::Vector4<f32>) {
        self.color = color;
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(self.color.x, self.color.y, self.color.z, self.color.w);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
