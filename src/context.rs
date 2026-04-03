use crate::{
    BufferType,
    buffer::Buffer,
    program::Program,
    render_context::RenderContext,
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayout},
    window_id::WindowId,
};

/// A GPU context object, containing a `wgpu::Instance` and `wgpu::Device`.
///
/// This struct implements methods used for GPU operations, acting as a sort of "central hub" for GPU access and usage.
pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Context {
    /// Constructs a new `Context` object with a primary-preferred backend (Vulkan, Metal, OpenGL, or DX12).
    ///
    /// This function is **thread-blocking**, as gaining access to GPU devices is not instantaneous.
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

    /// Constructs a new `Buffer` object, where `data` is an array of `f32` types.
    ///
    /// Buffer objects are wgpu objects that store an array of unformatted memory allocated on the GPU, and are used for
    /// GPU-based data allocations - such as vertex information, image pixels, etc.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, BufferType};
    ///
    /// let ctx = Context::new();
    /// let buffer = ctx.buffer(&[0.0, 0.0, 0.0], BufferType::Vertex);
    /// ```
    pub fn buffer(&self, data: &[f32], buffer_type: BufferType) -> Buffer {
        let byte_size = (data.len() * std::mem::size_of::<f32>()) as u64;

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_size,
            usage: match buffer_type {
                BufferType::Vertex => {
                    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                }
                BufferType::Index => {
                    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                }
                BufferType::Storage => {
                    wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                }
                BufferType::CopyDst => wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                BufferType::CopySrc => wgpu::BufferUsages::COPY_SRC,
            },
            mapped_at_creation: false,
        });

        // Upload data
        self.queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(data));

        Buffer::new(buffer)
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

    /// Read data from a referenced `Buffer` object.
    ///
    /// This function is **thread-blocking**, as reading data from the GPU to the CPU is a slow, inefficient process.
    /// Only recommended for compute-use and not render loops or graphics-heavy work.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, BufferType};
    ///
    /// let ctx = Context::new();
    /// let buffer = ctx.buffer(&[0.0, 0.0, 0.0], BufferType::Vertex);
    ///
    /// assert_eq!(vec![0.0, 0.0, 0.0], ctx.read_buffer(&buffer));
    /// ```
    pub fn read_buffer(&self, buffer: &Buffer) -> Vec<f32> {
        let size = buffer.inner().size();

        // create staging buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // encode copy
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_buffer_to_buffer(buffer.inner(), 0, &staging_buffer, 0, size);

        self.queue.submit(Some(encoder.finish()));

        // map buffer
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();

        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        });
        rx.recv().unwrap().unwrap();

        // read data
        let data = buffer_slice.get_mapped_range();

        let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

        result
    }

    /// Write `data` to a referenced `Buffer` object.
    ///
    /// # Errors
    ///
    /// This function will return `Err` if the `Buffer` object does not contain the `BufferType::CopyDst` usage.
    /// By default, vertex, index, and storage buffers contain `BufferType::CopyDst`.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, BufferType};
    ///
    /// let ctx = Context::new();
    /// let buffer = ctx.buffer(&[0.0, 0.0, 0.0], BufferType::Vertex);
    ///
    /// ctx.write_buffer(&buffer, &[1.0, 1.0, 1.0]);
    /// ```
    pub fn write_buffer(&self, buffer: &Buffer, data: &[f32]) -> Result<(), &'static str> {
        if !buffer
            .inner()
            .usage()
            .contains(wgpu::BufferUsages::COPY_DST)
        {
            return Err("Buffer must have COPY_DST usage");
        }

        self.queue
            .write_buffer(buffer.inner(), 0, bytemuck::cast_slice(data));

        Ok(())
    }

    pub fn read_texture(&self, texture: &Texture) -> &[f32] {
        &[]
    }
}
