use crate::quad;
use crate::render_gl::{self, buffer, data};
use crate::resources::Resources;
use failure;
use gl;
use nalgebra as na;
use std::rc::Rc;

pub struct DropQuad {
    program: render_gl::Program,
    texture: Rc<render_gl::Texture>,
    program_model_location: Option<i32>,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    texture_location: Option<i32>,
    resolution_location: Option<i32>,
    center_position_location: Option<i32>,
    pub quad: quad::Quad,
}

impl DropQuad {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        texture: Rc<render_gl::Texture>,
    ) -> Result<DropQuad, failure::Error> {
        // set up shader program
        let program = render_gl::Program::from_res(gl, res, "shaders/drop")?;

        let program_model_location = program.get_uniform_location("Model");
        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let texture_location = program.get_uniform_location("Texture");
        let resolution_location = program.get_uniform_location("Resolution");
        let center_position_location = program.get_uniform_location("CenterPosition");

        let quad = quad::Quad::new(gl)?;

        Ok(DropQuad {
            texture,
            program,
            program_model_location,
            program_view_location,
            program_projection_location,
            texture_location,
            resolution_location,
            center_position_location,
            quad,
        })
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        model_matrix: &na::Matrix4<f32>,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        resolution: &na::Vector2<f32>,
        center_position: &na::Vector3<f32>,
    ) {
        self.program.set_used();

        if let Some(loc) = self.texture_location {
            self.texture.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let Some(loc) = self.resolution_location {
            self.program.set_uniform_2f(loc, resolution);
        }

        if let Some(loc) = self.center_position_location {
            self.program.set_uniform_3f(loc, center_position);
        }

        if let Some(loc) = self.program_model_location {
            self.program.set_uniform_matrix_4fv(loc, model_matrix);
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
