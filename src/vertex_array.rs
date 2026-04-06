pub enum VertexAttributeFormat {
    // floats
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    // unsigned ints
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,

    // signed ints
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,

    // normalized
    Unorm8x2,
    Unorm8x4,
    Snorm8x2,
    Snorm8x4,

    // optional
    Float16x2,
    Float16x4,
}

impl VertexAttributeFormat {
    pub fn to_wgpu(&self) -> wgpu::VertexFormat {
        match self {
            Self::Float32 => wgpu::VertexFormat::Float32,
            Self::Float32x2 => wgpu::VertexFormat::Float32x2,
            Self::Float32x3 => wgpu::VertexFormat::Float32x3,
            Self::Float32x4 => wgpu::VertexFormat::Float32x4,
            Self::Uint32 => wgpu::VertexFormat::Uint32,
            Self::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            Self::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            Self::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            Self::Sint32 => wgpu::VertexFormat::Sint32,
            Self::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            Self::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            Self::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            Self::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            Self::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
            Self::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            Self::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            Self::Float16x2 => wgpu::VertexFormat::Float16x2,
            Self::Float16x4 => wgpu::VertexFormat::Float16x4,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::Float32 => 4,
            Self::Float32x2 => 8,
            Self::Float32x3 => 12,
            Self::Float32x4 => 16,
            Self::Uint32 => 4,
            Self::Uint32x2 => 8,
            Self::Uint32x3 => 12,
            Self::Uint32x4 => 16,
            Self::Sint32 => 4,
            Self::Sint32x2 => 8,
            Self::Sint32x3 => 12,
            Self::Sint32x4 => 16,
            Self::Unorm8x2 => 2,
            Self::Unorm8x4 => 4,
            Self::Snorm8x2 => 2,
            Self::Snorm8x4 => 4,
            Self::Float16x2 => 4,
            Self::Float16x4 => 8,
        }
    }
}

pub(crate) struct VertexAttribute {
    pub(crate) location: u32,
    pub(crate) format: VertexAttributeFormat,
}

pub struct VertexLayout {
    pub(crate) attributes: Vec<VertexAttribute>,
}

impl VertexLayout {
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }

    pub fn attr(mut self, location: u32, format: VertexAttributeFormat) -> Self {
        self.attributes.push(VertexAttribute { location, format });

        self
    }
}

pub struct VertexArray {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: Option<wgpu::Buffer>,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
}
