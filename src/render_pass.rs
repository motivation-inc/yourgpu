use crate::{
    DepthConfig, StencilConfig, buffer::Buffer, texture::Texture, vertex_array::VertexArray,
};

/// Describes the cull mode used on a render pass.
pub enum RenderCullMode {
    Front,
    Back,
}

impl RenderCullMode {
    pub(crate) fn to_wgpu(&self) -> wgpu::Face {
        match self {
            RenderCullMode::Front => wgpu::Face::Front,
            RenderCullMode::Back => wgpu::Face::Back,
        }
    }
}

/// Describes the front face mode used on a render pass.
pub enum RenderFrontFaceMode {
    Clockwise,
    CounterClockwise,
}

impl RenderFrontFaceMode {
    pub(crate) fn to_wgpu(&self) -> wgpu::FrontFace {
        match self {
            RenderFrontFaceMode::Clockwise => wgpu::FrontFace::Cw,
            RenderFrontFaceMode::CounterClockwise => wgpu::FrontFace::Ccw,
        }
    }
}

pub(crate) enum RenderOperation<'a> {
    Draw(&'a VertexArray),
    SetCullMode(Option<wgpu::Face>),
    SetFrontFace(wgpu::FrontFace),
    SetDepthConfig(Option<DepthConfig>),
    SetStencilConfig(Option<StencilConfig>),
    SetStencilReference(u32),
    SetViewport(f32, f32, f32, f32, f32, f32),
    SetScissorRect(u32, u32, u32, u32),
    SetBuffer(String, &'a Buffer),
    SetTexture(String, &'a Texture),

    // compute specific
    DispatchWorkgroups(u32, u32, u32),
}

/// A single render pass containing render operations.
///
/// `RenderPass` objects are sequential, meaning any operations (e.g. `clear()`, `draw()`, etc.) are rendered
/// in order.
///
/// Created using `Context::render_texture` or `Context::render_window`.
pub struct RenderPass<'a> {
    pub(crate) clear: wgpu::Color,
    pub(crate) operations: Vec<RenderOperation<'a>>,
}

impl<'a> RenderPass<'a> {
    /// Clear operation filling the screen with the specified RGBA data.
    ///
    /// The last clear operation is used by default, as only one clear operation
    /// is allowed per render pass.
    pub fn clear(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear = wgpu::Color { r, g, b, a }
    }

    /// Draw operation for a `VertexArray` object.
    ///
    /// - `vao`: the vertex array to draw
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

    /// Set cull mode operation.
    ///
    /// - `cull_mode`: the `RenderCullMode` to use
    pub fn set_cull_mode(&mut self, cull_mode: Option<RenderCullMode>) {
        match cull_mode {
            Some(c) => self
                .operations
                .push(RenderOperation::SetCullMode(Some(c.to_wgpu()))),
            None => self.operations.push(RenderOperation::SetCullMode(None)),
        }
    }

    /// Set front face operation.
    ///
    /// - `front_face`: the `RenderFrontFaceMode` to use
    pub fn set_front_face(&mut self, front_face: RenderFrontFaceMode) {
        self.operations
            .push(RenderOperation::SetFrontFace(front_face.to_wgpu()));
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

    /// Set depth configuration operation.
    ///
    /// - `config`: the depth buffer configuration
    pub fn set_depth_config(&mut self, config: Option<DepthConfig>) {
        self.operations
            .push(RenderOperation::SetDepthConfig(config));
    }

    /// Set stencil configuration operation.
    ///
    /// - `config`: the stencil buffer configuration
    pub fn set_stencil_config(&mut self, config: Option<StencilConfig>) {
        self.operations
            .push(RenderOperation::SetStencilConfig(config));
    }

    /// Set stencil reference operation.
    ///
    /// - `reference`: the stencil to use
    ///
    /// Subsequent stencil tests will test against this value. If this method has not been called,
    /// the stencil reference value defaults to 0.
    pub fn set_stencil_reference(&mut self, reference: u32) {
        self.operations
            .push(RenderOperation::SetStencilReference(reference));
    }

    /// Set buffer operation.
    ///
    /// - `name`: the program binding name
    /// - `buffer`: the data to set the buffer binding to
    pub fn set_buffer(&mut self, name: &str, buffer: &'a Buffer) {
        self.operations
            .push(RenderOperation::SetBuffer(name.to_string(), buffer));
    }

    /// Set texture operation.
    ///
    /// - `name`: the program binding name
    /// - `texture`: the data to set the texture binding to
    pub fn set_texture(&mut self, name: &str, texture: &'a Texture) {
        self.operations
            .push(RenderOperation::SetTexture(name.to_string(), texture));
    }

    /// Dispatch workgroup operation.
    ///
    /// This function is reserved for use in compute programs with `Context::compute` and will be ignored
    /// if used otherwise.
    ///
    /// - `x`: the x dimension workgroup size
    /// - `y`: the y dimension workgroup size
    /// - `z`: the z dimension workgroup size
    pub fn dispatch_workgroups(&mut self, x: u32, y: u32, z: u32) {
        self.operations
            .push(RenderOperation::DispatchWorkgroups(x, y, z));
    }
}
