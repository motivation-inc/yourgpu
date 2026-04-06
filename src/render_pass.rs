use crate::{bind_group::BindGroup, vertex_array::VertexArray};

pub(crate) enum RenderOperation<'a> {
    Clear(f64, f64, f64, f64),
    Draw(&'a VertexArray, Vec<&'a BindGroup>),
}

/// A single render pass containing render operations.
///
/// Created using `Context::render_texture` or `Context::render_window`.
pub struct RenderPass<'a> {
    pub(crate) operations: Vec<RenderOperation<'a>>,
}

impl<'a> RenderPass<'a> {
    /// Clear operation filling the screen with the specified RGBA data.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType};
    ///
    /// let ctx = Context::new();
    /// let tex = ctx.texture(
    ///     1028,
    ///     1028,
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    /// );
    ///
    /// ctx.render_texture(&tex, |r| {
    ///     r.clear(0.0, 0.0, 0.0, 1.0) // solid black
    /// })
    /// ```
    pub fn clear(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.operations.push(RenderOperation::Clear(r, g, b, a));
    }

    /// Draw operation for the `vao` object (`VertexArray`) and specified `bind_groups`.
    ///
    /// Each bind group in `bind_groups` is bound to the render pass.
    ///
    /// # Example
    ///
    /// ```
    /// panic!("UNIMPLEMENTED");
    /// ```
    pub fn draw(&mut self, vao: &'a VertexArray, bind_groups: Vec<&'a BindGroup>) {
        self.operations
            .push(RenderOperation::Draw(&vao, bind_groups));
    }
}
