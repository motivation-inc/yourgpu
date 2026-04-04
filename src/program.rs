pub struct Program {
    vertex_shader: wgpu::ShaderModule,
    fragment_shader: Option<wgpu::ShaderModule>,
}

impl Program {
    pub fn new(
        vertex_shader: wgpu::ShaderModule,
        fragment_shader: Option<wgpu::ShaderModule>,
    ) -> Self {
        Self {
            vertex_shader,
            fragment_shader,
        }
    }

    pub fn vertex_shader(&self) -> &wgpu::ShaderModule {
        &self.vertex_shader
    }

    pub fn fragment_shader(&self) -> Option<&wgpu::ShaderModule> {
        self.fragment_shader.as_ref()
    }
}
