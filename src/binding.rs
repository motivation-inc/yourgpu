use std::collections::HashMap;

/// Describes how a shader program's bindings should be laid out.
pub struct BindingBuilder {
    pub(crate) entries: HashMap<String, wgpu::BindGroupLayoutEntry>,
}

impl BindingBuilder {
    /// Constructs a new `BindingBuilder`.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::BindingBuilder;
    ///
    /// let bindings = BindingBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Corresponds to a WGSL `uniform` binding.
    ///
    /// - `name`: the binding name
    /// - `binding`: the binding location
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::BindingBuilder;
    ///
    /// let bindings = BindingBuilder::new().uniform("u_color", 0);
    /// ```
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

    /// Corresponds to a WGSL `texture_2d` binding.
    ///
    /// - `name`: the binding name.
    /// - `binding`: the binding location
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::BindingBuilder;
    ///
    /// let bindings = BindingBuilder::new().texture_2d("tex", 0);
    /// ```
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

    /// Corresponds to a WGSL `texture_3d` binding.
    ///
    /// - `name`: the binding name
    /// - `binding`: the binding location
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::BindingBuilder;
    ///
    /// let bindings = BindingBuilder::new().texture_3d("tex", 0);
    /// ```
    pub fn texture_3d(mut self, name: &str, binding: u32) -> Self {
        self.entries.insert(
            name.to_string(),
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D3,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        );

        self
    }

    /// Corresponds to a WGSL `texture_cube` binding.
    ///
    /// - `name`: the binding name
    /// - `binding`: the binding location
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::BindingBuilder;
    ///
    /// let bindings = BindingBuilder::new().texture_cube("tex", 0);
    /// ```
    pub fn texture_cube(mut self, name: &str, binding: u32) -> Self {
        self.entries.insert(
            name.to_string(),
            wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::Cube,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
        );

        self
    }
}
