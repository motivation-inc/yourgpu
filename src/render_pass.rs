use crate::{buffer::Buffer, texture::Texture, vertex_array::VertexArray};

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

/// Describes the comparison mode used for depth and stencil operations on a render pass.
pub enum RenderDepthComparison {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl RenderDepthComparison {
    pub(crate) fn to_wgpu(&self) -> wgpu::CompareFunction {
        match self {
            RenderDepthComparison::Never => wgpu::CompareFunction::Never,
            RenderDepthComparison::Less => wgpu::CompareFunction::Less,
            RenderDepthComparison::Equal => wgpu::CompareFunction::Equal,
            RenderDepthComparison::LessEqual => wgpu::CompareFunction::LessEqual,
            RenderDepthComparison::Greater => wgpu::CompareFunction::Greater,
            RenderDepthComparison::NotEqual => wgpu::CompareFunction::NotEqual,
            RenderDepthComparison::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            RenderDepthComparison::Always => wgpu::CompareFunction::Always,
        }
    }
}

pub(crate) enum RenderOperation<'a> {
    Clear(f64, f64, f64, f64),
    Draw(&'a VertexArray),
    SetCullMode(Option<wgpu::Face>),
    SetFrontFace(wgpu::FrontFace),
    SetDepthTest(bool, wgpu::CompareFunction),
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

    /// Set cull mode operation, where `cull_mode` is the `RenderCullMode` to use.
    pub fn set_cull_mode(&mut self, cull_mode: Option<RenderCullMode>) {
        match cull_mode {
            Some(c) => self
                .operations
                .push(RenderOperation::SetCullMode(Some(c.to_wgpu()))),
            None => self.operations.push(RenderOperation::SetCullMode(None)),
        }
    }

    /// Set front face operation, where `front_face` is the `RenderFrontFaceMode` to use.
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

    /// Set depth test operation, where `write` is if the depth is write enabled, and `depth_compare` is the
    /// depth comparison.
    pub fn set_depth_test(&mut self, write: bool, depth_compare: RenderDepthComparison) {
        self.operations.push(RenderOperation::SetDepthTest(
            write,
            depth_compare.to_wgpu(),
        ));
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
