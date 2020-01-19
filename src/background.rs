use crate::quad;
use crate::render_gl::{self, buffer, data};
use crate::resources::Resources;
use failure;
use gl;
use nalgebra as na;
use std::rc::Rc;

pub struct Background {
    program: render_gl::Program,
    texture: Rc<render_gl::Texture>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    texture_location: Option<i32>,
    resolution_location: Option<i32>,
    quad: quad::Quad,
}

impl Background {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        texture: Rc<render_gl::Texture>,
        screen_width: u32,
        screen_height: u32,
    ) -> Result<Background, failure::Error> {
        // set up shader program
        let program = render_gl::Program::from_res(gl, res, "shaders/background")?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let texture_location = program.get_uniform_location("Texture");
        let resolution_location = program.get_uniform_location("Resolution");

        let texture_dimensions = texture.dimensions();

        let ratios = (
            screen_width as f32 / texture_dimensions.0 as f32,
            screen_height as f32 / texture_dimensions.1 as f32,
        );

        let target_dimensions = if ratios.0 < ratios.1 {
            (texture_dimensions.0 as f32 * ratios.1, screen_height as f32)
        } else {
            (screen_width as f32, texture_dimensions.1 as f32 * ratios.0)
        };

        let offsets = (
            (target_dimensions.0 - screen_width as f32) / 2.0,
            (target_dimensions.1 - screen_height as f32) / 2.0,
        );

        let bottom = -offsets.1;
        let left = -offsets.0;
        let top = target_dimensions.1 - offsets.1;
        let right = target_dimensions.0 - offsets.0;

        let quad = quad::Quad::new_with_size(gl, bottom, left, top, right)?;

        Ok(Background {
            texture,
            program,
            program_view_location,
            program_projection_location,
            texture_location,
            resolution_location,
            quad,
        })
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        resolution: &na::Vector2<f32>,
    ) {
        self.program.set_used();

        if let Some(loc) = self.texture_location {
            self.texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let Some(loc) = self.resolution_location {
            self.program.set_uniform_2f(loc, resolution);
        }

        if let Some(loc) = self.program_view_location {
            self.program.set_uniform_matrix_4fv(loc, view_matrix);
        }

        if let Some(loc) = self.program_projection_location {
            self.program.set_uniform_matrix_4fv(loc, proj_matrix);
        }

        self.quad.render(gl);
    }
}
