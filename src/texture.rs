/// Describes the format of the texture data.
pub enum TextureFormat {
    // color
    Rgba8Unorm,
    Rgba8UnormSrgb,
    R8Unorm,
    Rg8Unorm,

    // float
    Rgba16Float,
    Rgba32Float,

    // depth
    Depth24Plus,
    Depth32Float,

    // depth + stencil
    Depth24PlusStencil8,
    Depth32FloatStencil8,

    // integer
    Rgba8Uint,
    Rgba8Sint,
}

impl TextureFormat {
    pub(crate) fn to_wgpu(&self) -> wgpu::TextureFormat {
        match self {
            // color
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            TextureFormat::Rg8Unorm => wgpu::TextureFormat::Rg8Unorm,
            TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
            TextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
            TextureFormat::Depth24PlusStencil8 => wgpu::TextureFormat::Depth24PlusStencil8,
            TextureFormat::Depth32FloatStencil8 => wgpu::TextureFormat::Depth32FloatStencil8,
            TextureFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            TextureFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Rgba8Uint
            | TextureFormat::Rgba8Sint => 4,
            TextureFormat::R8Unorm => 1,
            TextureFormat::Rg8Unorm => 2,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
            TextureFormat::Depth24Plus => 4, // padded to 32-bit
            TextureFormat::Depth32Float => 4,
            TextureFormat::Depth24PlusStencil8 => 8,
            TextureFormat::Depth32FloatStencil8 => 8,
        }
    }
}

/// Describes the texture type.
pub enum TextureType {
    RenderAttachment,
    TextureBinding,
}

/// Describes the texture dimension.
pub enum TextureDimension {
    TwoDimensional,
    TwoDimensionalArray,
    ThreeDimensional,
    Cube,
    CubeArray,
}

impl TextureDimension {
    pub(crate) fn to_wgpu(&self) -> wgpu::TextureViewDimension {
        match self {
            TextureDimension::TwoDimensional => wgpu::TextureViewDimension::D2,
            TextureDimension::TwoDimensionalArray => wgpu::TextureViewDimension::D2Array,
            TextureDimension::ThreeDimensional => wgpu::TextureViewDimension::D3,
            TextureDimension::Cube => wgpu::TextureViewDimension::Cube,
            TextureDimension::CubeArray => wgpu::TextureViewDimension::CubeArray,
        }
    }
}

/// A texture object.
///
/// Created using `Context::texture`.
pub struct Texture {
    pub(crate) id: usize,
    pub(crate) format: TextureFormat,
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) height: u32,
    pub(crate) width: u32,
}
