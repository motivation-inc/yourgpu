use wgpu::wgc::pipeline::PipelineCache;
use winit::window::Window;

use crate::frame::Frame;

pub struct Context<'a> {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'a>>,
    pipeline_cache: PipelineCache,
}

impl<'a> Context<'a> {
    pub fn headless() {}
    pub fn from_window(window: &Window) {}
    pub fn vertex_buffer() {}
    pub fn index_buffer() {}

    pub fn frame() {}
}
