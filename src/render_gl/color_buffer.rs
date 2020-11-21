use gl;
use nalgebra as na;

pub struct ColorBuffer {
    pub color: na::Vector4<f32>,
}

impl ColorBuffer {
    pub fn from_rgba(color: na::Vector4<f32>) -> ColorBuffer {
        ColorBuffer { color }
    }

    pub fn from_rgb(color: na::Vector3<f32>) -> ColorBuffer {
        Self::from_rgba(color.fixed_resize::<na::U4, na::U1>(1.0))
    }

    pub fn update_color(&mut self, color: na::Vector3<f32>) {
        self.color = color.fixed_resize::<na::U4, na::U1>(1.0);
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
