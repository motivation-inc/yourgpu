/// Describes the comparison function used for depth and stencil operations.
pub enum Comparison {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl Comparison {
    pub(crate) fn to_wgpu(&self) -> wgpu::CompareFunction {
        match self {
            Comparison::Never => wgpu::CompareFunction::Never,
            Comparison::Less => wgpu::CompareFunction::Less,
            Comparison::Equal => wgpu::CompareFunction::Equal,
            Comparison::LessEqual => wgpu::CompareFunction::LessEqual,
            Comparison::Greater => wgpu::CompareFunction::Greater,
            Comparison::NotEqual => wgpu::CompareFunction::NotEqual,
            Comparison::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            Comparison::Always => wgpu::CompareFunction::Always,
        }
    }
}

/// Describes stencil operations for stencil values.
pub enum StencilOperation {
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

impl StencilOperation {
    pub(crate) fn to_wgpu(&self) -> wgpu::StencilOperation {
        match self {
            StencilOperation::Keep => wgpu::StencilOperation::Keep,
            StencilOperation::Zero => wgpu::StencilOperation::Zero,
            StencilOperation::Replace => wgpu::StencilOperation::Replace,
            StencilOperation::IncrementClamp => wgpu::StencilOperation::IncrementClamp,
            StencilOperation::DecrementClamp => wgpu::StencilOperation::DecrementClamp,
            StencilOperation::Invert => wgpu::StencilOperation::Invert,
            StencilOperation::IncrementWrap => wgpu::StencilOperation::IncrementWrap,
            StencilOperation::DecrementWrap => wgpu::StencilOperation::DecrementWrap,
        }
    }
}

/// A depth buffer configuration, used for configuring depth buffers.
#[derive(Clone, Copy)]
pub struct DepthConfig {
    pub(crate) write: bool,
    pub(crate) compare: wgpu::CompareFunction,
}

impl DepthConfig {
    /// Constructs a new depth configuration.
    ///
    /// - `write_enabled`: if the depth buffer can be written to
    /// - `comparison`: depth test comparison function
    pub fn new(write_enabled: bool, comparison: Comparison) -> Self {
        Self {
            write: write_enabled,
            compare: comparison.to_wgpu(),
        }
    }
}

/// A stencil buffer face configuration, used for configuring stencil buffer operations and comparison
/// functions.
pub struct StencilFaceConfig {
    pub(crate) compare: wgpu::CompareFunction,
    pub(crate) fail_op: wgpu::StencilOperation,
    pub(crate) depth_fail_op: wgpu::StencilOperation,
    pub(crate) pass_op: wgpu::StencilOperation,
}

impl StencilFaceConfig {
    /// Constructs a new stencil face configuration.
    ///
    /// - `comparison`: stencil test comparison function
    /// - `fail_op`: operation applied when the stencil test fails
    /// - `depth_fail_op`: operation applied when the stencil test passes but the depth test fails
    /// - `pass_op`: operation applied when both stencil and depth tests pass
    pub fn new(
        comparison: Comparison,
        fail_op: StencilOperation,
        depth_fail_op: StencilOperation,
        pass_op: StencilOperation,
    ) -> Self {
        Self {
            compare: comparison.to_wgpu(),
            fail_op: fail_op.to_wgpu(),
            depth_fail_op: fail_op.to_wgpu(),
            pass_op: fail_op.to_wgpu(),
        }
    }

    pub(crate) fn to_wgpu(&self) -> wgpu::StencilFaceState {
        wgpu::StencilFaceState {
            compare: self.compare,
            fail_op: self.fail_op,
            depth_fail_op: self.depth_fail_op,
            pass_op: self.pass_op,
        }
    }
}

/// A stencil buffer configuration, used for configuring stencil buffers.
#[derive(Clone, Copy)]
pub struct StencilConfig {
    pub(crate) front: wgpu::StencilFaceState,
    pub(crate) back: wgpu::StencilFaceState,
    pub(crate) read_mask: u32,
    pub(crate) write_mask: u32,
}

impl StencilConfig {
    /// Constructs a new stencil configuration.
    ///
    /// - `front_config`: stencil operations applied to front-facing fragments
    /// - `back_config`: stencil operations applied to back-facing fragments
    /// - `read_mask`: bitmask applied when reading stencil values during comparison
    /// - `write_mask`: bitmask controlling which bits can be written to the stencil buffer
    ///
    /// The stencil test is performed per-fragment using the configured comparison
    /// functions and operations defined in each face configuration.
    pub fn new(
        front_config: StencilFaceConfig,
        back_config: StencilFaceConfig,
        read_mask: u32,
        write_mask: u32,
    ) -> Self {
        Self {
            front: front_config.to_wgpu(),
            back: back_config.to_wgpu(),
            read_mask: read_mask,
            write_mask: write_mask,
        }
    }

    pub(crate) fn to_wgpu(&self) -> wgpu::StencilState {
        wgpu::StencilState {
            front: self.front,
            back: self.back,
            read_mask: self.read_mask,
            write_mask: self.write_mask,
        }
    }
}
