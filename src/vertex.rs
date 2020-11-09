use crate::render_gl::data::*;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = "0"]
    pub pos: f32_f32_f32,
    #[location = "1"]
    pub clr: f32_f32_f32_f32,
    #[location = "2"]
    pub uv: f32_f32,
}
