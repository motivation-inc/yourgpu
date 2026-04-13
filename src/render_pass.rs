use crate::{buffer::Buffer, texture::Texture, vertex_array::VertexArray};

pub(crate) enum RenderOperation<'a> {
    Clear(f64, f64, f64, f64),
    Draw(&'a VertexArray),
    SetViewport(f32, f32, f32, f32, f32, f32),
    SetScissorRect(u32, u32, u32, u32),
    SetUniform(String, &'a Buffer),
    SetTexture(String, &'a Texture),
}

/// A single render pass containing render operations.
///
/// `RenderPass` objects are sequential, meaning any operations (e.g. `clear()`, `draw()`, etc.) are rendered
/// in order.
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

    /// Draw operation for a `VertexArray` object, where `vao` is the vertex array to draw.
    pub fn draw(&mut self, vao: &'a VertexArray) {
        self.operations.push(RenderOperation::Draw(&vao));
    }

    /// Set viewport operation, where anything described between the bounds defined by
    /// `x`, `y`, `width`, `height`, `min_depth`, and `max_depth` is rendered.
    ///
    /// Subsequent draw calls will only draw within this region. If this method has
    /// not been called, the viewport defaults to the entire bounds of the render target.
    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.operations.push(RenderOperation::SetViewport(
            x, y, width, height, min_depth, max_depth,
        ))
    }

    /// Set scissor rectangle operation, where anything described between the bounds defined by
    /// `x`, `y`, `width`, and `height` is rendered.
    ///
    /// Subsequent draw calls will discard any fragments that are outside the bounds of the scissor rectangle.
    /// If this method has not been called, the scissor rectangle defaults to the entire bounds of the render
    /// target.
    ///
    /// The `set_scissor_rect()` operation is much similar to `set_viewport()`, but it does not affect the
    /// coordinate system, only which fragments are removed.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.operations
            .push(RenderOperation::SetScissorRect(x, y, width, height))
    }

    /// Set uniform operation, where `name` is the program binding name, and `buffer` is the data to set the
    /// uniform binding to.
    pub fn set_uniform(&mut self, name: &str, buffer: &'a Buffer) {
        self.operations
            .push(RenderOperation::SetUniform(name.to_string(), buffer));
    }

    /// Set texture operation, where `name` is the program binding name, and `texture` is the data to set the
    /// texture binding to.
    pub fn set_texture(&mut self, name: &str, texture: &'a Texture) {
        self.operations
            .push(RenderOperation::SetTexture(name.to_string(), texture));
    }
}
