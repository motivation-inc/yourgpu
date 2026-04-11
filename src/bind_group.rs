use std::collections::HashMap;

/// Describes how a shader program's bind group should be laid out.
pub struct BindGroupLayoutBuilder {
    pub(crate) entries: HashMap<String, wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn uniform(mut self, name: &str, binding: u32) -> Self {
        self.entries.insert(
            name.to_string(),
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        );
        self
    }

    pub fn texture_2d(mut self, name: &str, binding: u32) -> Self {
        self.entries.insert(
            name.to_string(),
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        );
        self
    }

    pub fn sampler(mut self, name: &str, binding: u32) -> Self {
        self.entries.insert(
            name.to_string(),
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        );
        self
    }
}
