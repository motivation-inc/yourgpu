use crate::vertex_array::VertexArray;

pub enum RenderOperation<'a> {
    Clear(f64, f64, f64, f64),
    Draw(&'a VertexArray),
}

/// A single render pass containing render operations.
pub struct RenderPass<'a> {
    pub(crate) operations: Vec<RenderOperation<'a>>,
}

impl<'a> RenderPass<'a> {
    pub fn clear(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.operations.push(RenderOperation::Clear(r, g, b, a));
    }

    pub fn draw(&mut self, vao: &'a VertexArray) {
        self.operations.push(RenderOperation::Draw(&vao));
    }
}
