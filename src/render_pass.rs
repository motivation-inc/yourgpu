use crate::{program::Program, vertex_array::VertexArray};

pub enum RenderOperation<'a, T>
where
    T: bytemuck::Pod,
{
    Clear(f64, f64, f64, f64),
    Draw(&'a Program, &'a VertexArray),
    SetUniform(u32, &'a T),
}

/// A single render pass containing render operations.
pub struct RenderPass<'a, T>
where
    T: bytemuck::Pod,
{
    pub(crate) operations: Vec<RenderOperation<'a, T>>,
}

impl<'a, T: bytemuck::Pod> RenderPass<'a, T> {
    pub fn clear(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.operations.push(RenderOperation::Clear(r, g, b, a));
    }

    pub fn draw(&mut self, program: &'a Program, vao: &'a VertexArray) {
        self.operations.push(RenderOperation::Draw(program, vao));
    }

    pub fn set_uniform(&mut self, location: u32, data: &'a T) {
        self.operations
            .push(RenderOperation::SetUniform(location, data));
    }
}
