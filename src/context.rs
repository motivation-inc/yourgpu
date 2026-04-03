use wgpu::{BackendOptions, BufferUsages, GlBackendOptions, MemoryBudgetThresholds};

use crate::{
    buffer::Buffer,
    program::Program,
    render_context::RenderContext,
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayout},
    window_id::WindowId,
};

pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    /// Constructs a new GPU context object with a primary-preferred backend (Vulkan, Metal, OpenGL, or DX12).
    ///
    /// This function is thread-blocking, as gaining access to GPU devices is not instant.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let ctx = Context::new(); // blocking
    /// ```
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            ..Default::default()
        }))
        .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }

    /// Attaches a [winit `Window` object](https://docs.rs/winit-gtk/latest/winit/window/struct.Window.html) to the context, returning a `WindowId` object referencing the window.
    pub fn attach_window(&self) -> WindowId {
        WindowId(0)
    }

    pub fn program(&self, vertex_shader: &str, fragment_shader: Option<&str>) -> Program {
        Program {}
    }

    pub fn buffer(&self, data: &[f32]) -> Buffer {
        Buffer {}
    }

    pub fn texture(
        &self,
        width: usize,
        height: usize,
        data: &[f32],
        format: TextureFormat,
    ) -> Texture {
        Texture {}
    }

    pub fn vertex_array(
        &self,
        program: &Program,
        buffer: &Buffer,
        layout: VertexLayout,
    ) -> VertexArray {
        VertexArray {}
    }

    pub fn render_texture<F>(&self, texture: &Texture, f: F)
    where
        F: FnOnce(&mut RenderContext),
    {
        let mut r = RenderContext {};

        f(&mut r);
    }

    pub fn render_window<F>(&self, window_id: &WindowId, f: F)
    where
        F: FnOnce(&mut RenderContext),
    {
        let mut r = RenderContext {};

        f(&mut r);
    }

    pub fn read_texture(&self, texture: &Texture) -> &[f32] {
        &[]
    }
}
