/// A program object, containing vertex + fragment shaders, and a bind group layout.
pub struct Program {
    pub(crate) vertex_shader: wgpu::ShaderModule,
    pub(crate) fragment_shader: Option<wgpu::ShaderModule>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}
