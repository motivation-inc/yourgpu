use crate::{program::Program, vertex_array::VertexArray};

pub struct RenderContext {}

impl RenderContext {
    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {}
    pub fn draw(&self, program: &Program, vao: &VertexArray) {}
}
