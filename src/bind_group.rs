use crate::{buffer::Buffer, texture::Texture};

/// Describes where the bind group is visible during the shading stage.
pub enum BindGroupVisibilty {
    Vertex,
    Fragment,
    VertexFragment,
}

impl BindGroupVisibilty {
    pub(crate) fn to_wgpu(&self) -> wgpu::ShaderStages {
        match self {
            BindGroupVisibilty::Vertex => wgpu::ShaderStages::VERTEX,
            BindGroupVisibilty::Fragment => wgpu::ShaderStages::FRAGMENT,
            BindGroupVisibilty::VertexFragment => wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }
}

/// A bind group object representing the set of resources bound to the bindings described by a `BindGroupLayout`.
/// Bind groups can be bound to a particular `RenderPass`.
///
/// Created using `Context::bind_group`.
pub struct BindGroup {
    pub(crate) bind_group: wgpu::BindGroup,
}

/// A bind group layout object representing the layout of resources bound to the bind group.
///
/// Created using `Context::bind_group_layout`.
pub struct BindGroupLayout {
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

/// A bind group layout builder object describing the bindings of a `BindGroup` object.
pub struct BindGroupLayoutBuilder {
    pub(crate) entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn uniform(mut self, binding: u32, visibility: BindGroupVisibilty) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: visibility.to_wgpu(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
        self
    }

    pub fn texture_2d(mut self, binding: u32, visibility: BindGroupVisibilty) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: visibility.to_wgpu(),
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });
        self
    }

    pub fn sampler(mut self, binding: u32, visibility: BindGroupVisibilty) -> Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility: visibility.to_wgpu(),
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });
        self
    }
}

/// A bind group builder object, describing how to build the bindings of a `BindGroup` object.
pub struct BindGroupBuilder<'a> {
    pub(crate) entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn uniform(mut self, binding: u32, buffer: &'a Buffer) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: buffer.buffer.as_entire_binding(),
        });
        self
    }

    pub fn texture(mut self, binding: u32, texture: &'a Texture) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });
        self
    }

    pub fn sampler(mut self, binding: u32, texture: &'a Texture) -> Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(&texture.sampler),
        });
        self
    }
}
