use crate::{buffer::Buffer, texture::Texture, vertex_array::VertexArray};

pub(crate) enum RenderOperation<'a> {
    Clear(f64, f64, f64, f64),
    Draw(&'a VertexArray),
    SetUniform(String, &'a Buffer),
    SetTexture(String, &'a Texture),
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

    /// Draw operation for `vao` (a `VertexArray` object.)
    pub fn draw(&mut self, vao: &'a VertexArray) {
        self.operations.push(RenderOperation::Draw(&vao));
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
