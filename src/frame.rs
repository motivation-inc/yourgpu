use crate::context::Context;

pub struct Frame<'a> {
    encoder: wgpu::CommandEncoder,
    view: wgpu::TextureView,
    ctx: &'a Context<'a>,
}
