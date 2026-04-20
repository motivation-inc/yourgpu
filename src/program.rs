use std::collections::HashMap;

/// A shader program object.
///
/// Created using `Context::program`.
pub struct Program {
    pub(crate) id: usize,
    pub(crate) pipeline_layout_id: usize,
    pub(crate) bind_group_layout_id: usize,
    pub(crate) vertex_shader: wgpu::ShaderModule,
    pub(crate) fragment_shader: Option<wgpu::ShaderModule>,
    pub(crate) bind_group_layouts: HashMap<u32, wgpu::BindGroupLayout>, // (group, layout)
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) bind_group_entries: HashMap<u32, HashMap<String, wgpu::BindGroupLayoutEntry>>, // (group, (name, entry))
    pub(crate) entry_names: Vec<String>, // binding names
}
