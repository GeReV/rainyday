use crate::quad;
use crate::render_gl::{self, buffer, data};
use crate::resources::Resources;
use failure;
use gl;
use nalgebra as na;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
struct Vertex {
    #[location = "0"]
    pos: data::f32_f32_f32,
    #[location = "1"]
    clr: data::f32_f32_f32_f32,
    #[location = "2"]
    uv: data::f32_f32,
}

pub struct Background {
    program: render_gl::Program,
    texture: render_gl::Texture,
    program_view_location: Option<i32>,
    program_projection_location: Option<i32>,
    texture_location: Option<i32>,
    resolution_location: Option<i32>,
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
}

impl Background {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        screen_width: u32,
        screen_height: u32,
    ) -> Result<Background, failure::Error> {
        // set up shader program
        let texture = render_gl::Texture::from_res_rgb("textures/background.jpg").load(gl, res)?;

        let program = render_gl::Program::from_res(gl, res, "shaders/quad")?;

        let program_view_location = program.get_uniform_location("View");
        let program_projection_location = program.get_uniform_location("Projection");
        let texture_location = program.get_uniform_location("Texture");
        let resolution_location = program.get_uniform_location("Resolution");

        let texture_dimensions = texture.dimensions();

        println!("{:?}", texture_dimensions);

        let ratios = (
            screen_width as f32 / texture_dimensions.0 as f32,
            screen_height as f32 / texture_dimensions.1 as f32,
        );

        println!("{:?}", ratios);

        let target_dimensions = if ratios.0 < ratios.1 {
            (texture_dimensions.0 as f32 * ratios.1, screen_height as f32)
        } else {
            (screen_width as f32, texture_dimensions.1 as f32 * ratios.0)
        };

        println!("{:?}", target_dimensions);

        let offsets = (
            (target_dimensions.0 - screen_width as f32) / 2.0,
            (target_dimensions.1 - screen_height as f32) / 2.0,
        );

        println!("{:?}", offsets);

        let quad = quad::Quad::new_with_size(
            res,
            gl,
            target_dimensions.1 + offsets.1,
            -offsets.0,
            -offsets.1,
            target_dimensions.0 + offsets.0,
        )?;

        let bottom = target_dimensions.1 + offsets.1;
        let top = -offsets.1;
        let left = -offsets.0;
        let right = target_dimensions.0 + offsets.0;

        let v0 = (bottom, left, 0.0);
        let v1 = (top, left, 0.0);
        let v2 = (bottom, right, 0.0);
        let v3 = (top, right, 0.0);

        let white = (1.0, 1.0, 1.0, 1.0);

        let vbo_data = vec![
            Vertex {
                pos: v0.into(),
                clr: white.into(),
                uv: (0.0, 1.0).into(),
            }, // 0
            Vertex {
                pos: v1.into(),
                clr: white.into(),
                uv: (1.0, 1.0).into(),
            }, // 1
            Vertex {
                pos: v2.into(),
                clr: white.into(),
                uv: (0.0, 0.0).into(),
            }, // 2
            Vertex {
                pos: v3.into(),
                clr: white.into(),
                uv: (1.0, 0.0).into(),
            }, // 3
        ];

        let ebo_data: Vec<u8> = vec![0, 1, 2, 1, 3, 2];

        let vbo = buffer::ArrayBuffer::new(gl);
        vbo.bind();
        vbo.static_draw_data(&vbo_data);
        vbo.unbind();

        let ebo = buffer::ElementArrayBuffer::new(gl);
        ebo.bind();
        ebo.static_draw_data(&ebo_data);
        ebo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new(gl);

        vao.bind();
        vbo.bind();
        ebo.bind();
        Vertex::vertex_attrib_pointers(gl);
        vao.unbind();

        vbo.unbind();
        ebo.unbind();

        Ok(Background {
            texture,
            program,
            program_view_location,
            program_projection_location,
            texture_location,
            resolution_location,
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
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

        self.vao.bind();

        unsafe {
            gl.DrawElements(
                gl::TRIANGLES,      // mode
                self.index_count,   // index vertex count
                gl::UNSIGNED_BYTE,  // index type
                ::std::ptr::null(), // pointer to indices (we are using ebo configured at vao creation)
            );
        }
    }
}
