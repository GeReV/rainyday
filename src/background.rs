use crate::quad;
use crate::render_gl::{Error, FrameBuffer, Program, Shader, Texture};
use failure;
use gl;
use nalgebra as na;
use std::rc::Rc;

const BACKGROUND_VERT: &str = include_str!("../assets/shaders/background.vert");
const BACKGROUND_FRAG: &str = include_str!("../assets/shaders/background.frag");
const BACKGROUND_PASS_RED_FRAG: &str = include_str!("../assets/shaders/background_pass_red.frag");
const BACKGROUND_PASS_GREEN_FRAG: &str =
    include_str!("../assets/shaders/background_pass_green.frag");
const BACKGROUND_PASS_BLUE_FRAG: &str = include_str!("../assets/shaders/background_pass_blue.frag");

const _KERNEL0BRACKETS_REAL_XY_IM_ZW: [f32; 2] = [0.411259, -0.548794];
const KERNEL0_REAL_X_IM_Y_REAL_Z_IM_W: [[f32; 4]; 17] = [
    [
        /*XY: Non Bracketed*/ 0.014096, -0.022658, /*Bracketed WZ:*/ 0.055991, 0.004413,
    ],
    [
        /*XY: Non Bracketed*/ -0.020612, -0.025574, /*Bracketed WZ:*/ 0.019188, 0.000000,
    ],
    [
        /*XY: Non Bracketed*/ -0.038708, 0.006957, /*Bracketed WZ:*/ 0.000000, 0.049223,
    ],
    [
        /*XY: Non Bracketed*/ -0.021449, 0.040468, /*Bracketed WZ:*/ 0.018301, 0.099929,
    ],
    [
        /*XY: Non Bracketed*/ 0.013015, 0.050223, /*Bracketed WZ:*/ 0.054845, 0.114689,
    ],
    [
        /*XY: Non Bracketed*/ 0.042178, 0.038585, /*Bracketed WZ:*/ 0.085769, 0.097080,
    ],
    [
        /*XY: Non Bracketed*/ 0.057972, 0.019812, /*Bracketed WZ:*/ 0.102517, 0.068674,
    ],
    [
        /*XY: Non Bracketed*/ 0.063647, 0.005252, /*Bracketed WZ:*/ 0.108535, 0.046643,
    ],
    [
        /*XY: Non Bracketed*/ 0.064754, 0.000000, /*Bracketed WZ:*/ 0.109709, 0.038697,
    ],
    [
        /*XY: Non Bracketed*/ 0.063647, 0.005252, /*Bracketed WZ:*/ 0.108535, 0.046643,
    ],
    [
        /*XY: Non Bracketed*/ 0.057972, 0.019812, /*Bracketed WZ:*/ 0.102517, 0.068674,
    ],
    [
        /*XY: Non Bracketed*/ 0.042178, 0.038585, /*Bracketed WZ:*/ 0.085769, 0.097080,
    ],
    [
        /*XY: Non Bracketed*/ 0.013015, 0.050223, /*Bracketed WZ:*/ 0.054845, 0.114689,
    ],
    [
        /*XY: Non Bracketed*/ -0.021449, 0.040468, /*Bracketed WZ:*/ 0.018301, 0.099929,
    ],
    [
        /*XY: Non Bracketed*/ -0.038708, 0.006957, /*Bracketed WZ:*/ 0.000000, 0.049223,
    ],
    [
        /*XY: Non Bracketed*/ -0.020612, -0.025574, /*Bracketed WZ:*/ 0.019188, 0.000000,
    ],
    [
        /*XY: Non Bracketed*/ 0.014096, -0.022658, /*Bracketed WZ:*/ 0.055991, 0.004413,
    ],
];
const _KERNEL1WEIGHTS_REAL_X_IM_Y: [f32; 2] = [0.513282, 4.561110];
const KERNEL1_REAL_X_IM_Y_REAL_Z_IM_W: [[f32; 4]; 17] = [
    [
        /*XY: Non Bracketed*/ 0.000115, 0.009116, /*Bracketed WZ:*/ 0.000000, 0.051147,
    ],
    [
        /*XY: Non Bracketed*/ 0.005324, 0.013416, /*Bracketed WZ:*/ 0.009311, 0.075276,
    ],
    [
        /*XY: Non Bracketed*/ 0.013753, 0.016519, /*Bracketed WZ:*/ 0.024376, 0.092685,
    ],
    [
        /*XY: Non Bracketed*/ 0.024700, 0.017215, /*Bracketed WZ:*/ 0.043940, 0.096591,
    ],
    [
        /*XY: Non Bracketed*/ 0.036693, 0.015064, /*Bracketed WZ:*/ 0.065375, 0.084521,
    ],
    [
        /*XY: Non Bracketed*/ 0.047976, 0.010684, /*Bracketed WZ:*/ 0.085539, 0.059948,
    ],
    [
        /*XY: Non Bracketed*/ 0.057015, 0.005570, /*Bracketed WZ:*/ 0.101695, 0.031254,
    ],
    [
        /*XY: Non Bracketed*/ 0.062782, 0.001529, /*Bracketed WZ:*/ 0.112002, 0.008578,
    ],
    [
        /*XY: Non Bracketed*/ 0.064754, 0.000000, /*Bracketed WZ:*/ 0.115526, 0.000000,
    ],
    [
        /*XY: Non Bracketed*/ 0.062782, 0.001529, /*Bracketed WZ:*/ 0.112002, 0.008578,
    ],
    [
        /*XY: Non Bracketed*/ 0.057015, 0.005570, /*Bracketed WZ:*/ 0.101695, 0.031254,
    ],
    [
        /*XY: Non Bracketed*/ 0.047976, 0.010684, /*Bracketed WZ:*/ 0.085539, 0.059948,
    ],
    [
        /*XY: Non Bracketed*/ 0.036693, 0.015064, /*Bracketed WZ:*/ 0.065375, 0.084521,
    ],
    [
        /*XY: Non Bracketed*/ 0.024700, 0.017215, /*Bracketed WZ:*/ 0.043940, 0.096591,
    ],
    [
        /*XY: Non Bracketed*/ 0.013753, 0.016519, /*Bracketed WZ:*/ 0.024376, 0.092685,
    ],
    [
        /*XY: Non Bracketed*/ 0.005324, 0.013416, /*Bracketed WZ:*/ 0.009311, 0.075276,
    ],
    [
        /*XY: Non Bracketed*/ 0.000115, 0.009116, /*Bracketed WZ:*/ 0.000000, 0.051147,
    ],
];

pub struct Background {
    program: Program,
    texture: Rc<Texture>,
    mid_buffer_r: Texture,
    mid_program_r: Program,
    mid_buffer_g: Texture,
    mid_program_g: Program,
    mid_buffer_b: Texture,
    mid_program_b: Program,
    frame_buffer: FrameBuffer,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    resolution_location: Option<i32>,
    quad: quad::Quad,
    filter_radius: f32,
}

impl Background {
    pub fn new(
        gl: &gl::Gl,
        texture: Rc<Texture>,
        screen_width: u32,
        screen_height: u32,
        filter_radius: f32,
    ) -> Result<Background, failure::Error> {
        let program = Program::from_shaders(
            gl,
            &[
                Shader::from_vert_source_str(&gl, BACKGROUND_VERT)?,
                Shader::from_frag_source_str(&gl, BACKGROUND_FRAG)?,
            ],
        )
        .map_err(|msg| Error::LinkError {
            message: msg,
            name: "background".to_string(),
        })?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let resolution_location = program.get_uniform_location("Resolution");

        let quad =
            quad::Quad::new_with_size(gl, 0.0, 0.0, screen_height as f32, screen_width as f32);

        let mid_buffer_r = Texture::new(gl, screen_width, screen_height)?;
        let mid_buffer_g = Texture::new(gl, screen_width, screen_height)?;
        let mid_buffer_b = Texture::new(gl, screen_width, screen_height)?;

        let background_vert_shader_r = Shader::from_vert_source_str(&gl, BACKGROUND_VERT)?;
        let background_frag_shader_r = Shader::from_frag_source_str(gl, BACKGROUND_PASS_RED_FRAG)?;
        let background_vert_shader_g = Shader::from_vert_source_str(&gl, BACKGROUND_VERT)?;
        let background_frag_shader_g =
            Shader::from_frag_source_str(gl, BACKGROUND_PASS_GREEN_FRAG)?;
        let background_vert_shader_b = Shader::from_vert_source_str(&gl, BACKGROUND_VERT)?;
        let background_frag_shader_b = Shader::from_frag_source_str(gl, BACKGROUND_PASS_BLUE_FRAG)?;

        let mid_program_r =
            Program::from_shaders(gl, &[background_vert_shader_r, background_frag_shader_r])
                .map_err(|msg| Error::LinkError {
                    message: msg,
                    name: "mid_program_r".to_string(),
                })?;
        let mid_program_g =
            Program::from_shaders(gl, &[background_vert_shader_g, background_frag_shader_g])
                .map_err(|msg| Error::LinkError {
                    message: msg,
                    name: "mid_program_g".to_string(),
                })?;
        let mid_program_b =
            Program::from_shaders(gl, &[background_vert_shader_b, background_frag_shader_b])
                .map_err(|msg| Error::LinkError {
                    message: msg,
                    name: "mid_program_b".to_string(),
                })?;

        let frame_buffer = FrameBuffer::new(gl);

        Ok(Background {
            texture,
            program,
            mid_buffer_r,
            mid_program_r,
            mid_buffer_g,
            mid_program_g,
            mid_buffer_b,
            mid_program_b,
            frame_buffer,
            program_view_location,
            program_projection_location,
            resolution_location,
            quad,
            filter_radius,
        })
    }

    pub fn prepass(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        resolution: &na::Vector2<f32>,
    ) {
        self.render_pass(
            gl,
            &self.mid_program_r,
            &self.frame_buffer,
            &self.mid_buffer_r,
            view_matrix,
            proj_matrix,
            resolution,
        );

        self.render_pass(
            gl,
            &self.mid_program_g,
            &self.frame_buffer,
            &self.mid_buffer_g,
            view_matrix,
            proj_matrix,
            resolution,
        );

        self.render_pass(
            gl,
            &self.mid_program_b,
            &self.frame_buffer,
            &self.mid_buffer_b,
            view_matrix,
            proj_matrix,
            resolution,
        );
    }

    fn render_pass(
        &self,
        gl: &gl::Gl,
        program: &Program,
        frame_buffer: &FrameBuffer,
        texture_buffer: &Texture,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        resolution: &na::Vector2<f32>,
    ) {
        program.set_used();

        if let Some(loc) = program.get_uniform_location("Texture") {
            self.texture.bind_at(0);
            program.set_uniform_1i(loc, 0);
        }

        if let Some(loc) = program.get_uniform_location("Resolution") {
            program.set_uniform_2f(loc, resolution);
        }

        if let Some(loc) = program.get_uniform_location("View") {
            program.set_uniform_matrix_4fv(loc, view_matrix);
        }

        if let Some(loc) = program.get_uniform_location("Projection") {
            program.set_uniform_matrix_4fv(loc, proj_matrix);
        }

        if let Some(loc) = program.get_uniform_location("FilterRadius") {
            program.set_uniform_1f(loc, self.filter_radius)
        }

        if let Some(loc) = program.get_uniform_location("Kernel0") {
            program.set_uniform_4fv(loc, &KERNEL0_REAL_X_IM_Y_REAL_Z_IM_W[..])
        }
        if let Some(loc) = program.get_uniform_location("Kernel1") {
            program.set_uniform_4fv(loc, &KERNEL1_REAL_X_IM_Y_REAL_Z_IM_W[..])
        }

        frame_buffer.bind();
        frame_buffer.attach_texture(texture_buffer);

        self.quad.render(gl);

        frame_buffer.unbind();
    }

    pub fn render(
        &self,
        gl: &gl::Gl,
        view_matrix: &na::Matrix4<f32>,
        proj_matrix: &na::Matrix4<f32>,
        resolution: &na::Vector2<f32>,
    ) {
        self.program.set_used();

        if let Some(loc) = self.program.get_uniform_location("TextureR") {
            self.mid_buffer_r.bind_at(0);
            self.program.set_uniform_1i(loc, 0);
        }

        if let Some(loc) = self.program.get_uniform_location("TextureG") {
            self.mid_buffer_g.bind_at(1);
            self.program.set_uniform_1i(loc, 1);
        }

        if let Some(loc) = self.program.get_uniform_location("TextureB") {
            self.mid_buffer_b.bind_at(2);
            self.program.set_uniform_1i(loc, 2);
        }

        if let Some(loc) = self.program.get_uniform_location("Kernel0") {
            self.program
                .set_uniform_4fv(loc, &KERNEL0_REAL_X_IM_Y_REAL_Z_IM_W)
        }
        if let Some(loc) = self.program.get_uniform_location("Kernel1") {
            self.program
                .set_uniform_4fv(loc, &KERNEL1_REAL_X_IM_Y_REAL_Z_IM_W)
        }

        if let Some(loc) = self.program.get_uniform_location("FilterRadius") {
            self.program.set_uniform_1f(loc, self.filter_radius)
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
