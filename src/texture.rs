use crate::surface::Surface;

/// A texture format for describing various texture data.
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

    // integer
    Rgba8Uint,
    Rgba8Sint,
}

impl TextureFormat {
    pub fn to_wgpu(&self) -> wgpu::TextureFormat {
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
            TextureFormat::Rgba8Uint => wgpu::TextureFormat::Rgba8Uint,
            TextureFormat::Rgba8Sint => wgpu::TextureFormat::Rgba8Sint,
        }
    }
}

/// A texture type.
pub enum TextureType {
    RenderAttachment,
    TextureBinding,
}

pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl Texture {
    pub fn new(
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            texture,
            view,
            sampler,
            config,
        }
    }
}

impl Surface for Texture {
    fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
