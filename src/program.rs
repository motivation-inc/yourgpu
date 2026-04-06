/// A program object, containing vertex and fragment shaders.
pub struct Program {
    pub(crate) vertex_shader: wgpu::ShaderModule,
    pub(crate) fragment_shader: Option<wgpu::ShaderModule>,
}
