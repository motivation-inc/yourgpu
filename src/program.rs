use std::collections::HashMap;

/// A program object, containing vertex and fragment shaders.
pub struct Program {
    pub(crate) vertex_shader: wgpu::ShaderModule,
    pub(crate) fragment_shader: Option<wgpu::ShaderModule>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) bindings: HashMap<String, wgpu::BindGroupLayoutEntry>,
}
