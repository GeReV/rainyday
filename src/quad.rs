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

pub struct Quad {
    _vbo: buffer::ArrayBuffer,
    _ebo: buffer::ElementArrayBuffer,
    index_count: i32,
    vao: buffer::VertexArray,
}

impl Quad {
    pub fn new(gl: &gl::Gl) -> Result<Quad, failure::Error> {
        Quad::new_with_size(gl, -1.0, -1.0, 1.0, 1.0)
    }

    pub fn new_with_size(
        gl: &gl::Gl,
        bottom: f32,
        left: f32,
        top: f32,
        right: f32,
    ) -> Result<Quad, failure::Error> {
        let v0 = (left, bottom, 0.0);
        let v1 = (left, top, 0.0);
        let v2 = (right, bottom, 0.0);
        let v3 = (right, top, 0.0);

        let white = (1.0, 1.0, 1.0, 1.0);

        let vbo_data = vec![
            Vertex {
                pos: v0.into(),
                clr: white.into(),
                uv: (0.0, 0.0).into(),
            }, // 0
            Vertex {
                pos: v1.into(),
                clr: white.into(),
                uv: (0.0, 1.0).into(),
            }, // 1
            Vertex {
                pos: v2.into(),
                clr: white.into(),
                uv: (1.0, 0.0).into(),
            }, // 2
            Vertex {
                pos: v3.into(),
                clr: white.into(),
                uv: (1.0, 1.0).into(),
            }, // 3
        ];

        let ebo_data: Vec<u8> = vec![0, 2, 1, 1, 2, 3];

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

        Ok(Quad {
            _vbo: vbo,
            _ebo: ebo,
            index_count: ebo_data.len() as i32,
            vao,
        })
    }

    pub fn render(&self, gl: &gl::Gl) {
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
