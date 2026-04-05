use crate::{
    BufferType, TextureType,
    buffer::Buffer,
    program::Program,
    render_pass::{RenderOperation, RenderPass},
    surface::Surface,
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayout},
    window::WindowSurface,
};
use std::sync::Arc;
use winit::window::Window;

/// A GPU context object, containing a `wgpu::Instance` and `wgpu::Device`.
///
/// This struct implements methods used for GPU operations, acting as a sort of "central hub" for GPU access and usage.
pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> Context {
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

    /// Constructs a new `WindowSurface` object from a [winit `Window` object](https://docs.rs/winit-gtk/latest/winit/window/struct.Window.html).
    ///
    /// Since a winit `Window` is created asynchronously, `window` takes an `Arc<Window>` for thread safety.
    pub fn attach_window(&self, window: Arc<Window>) -> WindowSurface<'a> {
        let size = window.inner_size();
        let surface = self.instance.create_surface(window.clone()).unwrap();
        let surface_caps = surface.get_capabilities(&self.adapter);
        let surface_format = surface_caps.formats.iter().copied().find(|f| f.is_srgb());

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format.unwrap(),
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        WindowSurface {
            surface: surface,
            config: config,
        }
    }

    /// Constructs a new `Program` object, where `vertex_shader` is the vertex shader, and `fragment_shader` is the
    /// optional fragment shader contained in the program.
    ///
    /// This function requires the shader source to be valid [WebGPU Shading Language](https://www.w3.org/TR/WGSL/).
    ///
    /// # Example
    ///
    /// ```
    /// panic!("UNIMPLEMENTED");
    /// ```
    pub fn program(&self, vertex_shader: &str, fragment_shader: Option<&str>) -> Program {
        let vs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("vertex shader"),
                source: wgpu::ShaderSource::Wgsl(vertex_shader.into()),
            });
        let fs_module = fragment_shader.map(|fs| {
            self.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("fragment shader"),
                    source: wgpu::ShaderSource::Wgsl(fs.into()),
                })
        });

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[], // no uniforms yet
                });

        Program {
            vertex_shader: vs_module,
            fragment_shader: fs_module,
            bind_group_layout,
        }
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
        let bytes = bytemuck::cast_slice(data);
        let byte_size = (bytes.len() * std::mem::size_of::<u8>()) as u64;

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

        // upload data
        self.queue.write_buffer(&buffer, 0, bytes);

        Buffer {
            buffer,
            length: data.len() as u32,
        }
    }

    /// Constructs a new `Texture` object, with `width` and `height`, `data` being the image data (in bytes),
    /// and `format` being the texture format of the image.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType};
    ///
    /// let (width, height) = (2, 2);
    ///
    /// let ctx = Context::new();
    /// let tex = ctx.texture(width, height, &[0x32, 0x32, 0x32, 0x32], TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    /// ```
    pub fn texture(
        &self,
        width: u32,
        height: u32,
        bytes: &[u8],
        format: TextureFormat,
        texture_type: TextureType,
    ) -> Texture {
        let size = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.to_wgpu(),
            usage: match texture_type {
                TextureType::TextureBinding => {
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST
                }
                TextureType::RenderAttachment => {
                    wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::COPY_DST
                }
            },
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(format.bytes_per_pixel() * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::all(),
            format: format.to_wgpu(),
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::new(),
        };

        Texture {
            format,
            texture,
            view,
            sampler,
            config,
        }
    }

    pub fn vertex_array<T>(
        &self,
        surface: &T,
        program: &Program,
        vertex_buffer: &Buffer,
        index_buffer: Option<&Buffer>,
        layout: VertexLayout,
    ) -> VertexArray
    where
        T: Surface,
    {
        let mut offset = 0;
        let mut attrs = vec![];

        for attr in &layout.attributes {
            attrs.push(wgpu::VertexAttribute {
                offset,
                shader_location: attr.location,
                format: attr.format.to_wgpu(),
            });

            offset += attr.format.size();
        }

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: offset,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &attrs,
        };

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&program.bind_group_layout],
                immediate_size: 0,
            });

        let config = surface.config();

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &program.vertex_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_buffer_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &program.fragment_shader.as_ref().unwrap(),
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }), // TODO: remove option unwrap, fragment shader is never foolproof
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            });

        VertexArray {
            pipeline,
            vertex_buffer: vertex_buffer.buffer.clone(),
            index_buffer: index_buffer.map(|b| b.buffer.clone()),
            vertex_count: vertex_buffer.length,
            index_count: index_buffer.map(|b| b.length).unwrap_or(0),
        }
    }

    pub fn render_texture<F>(&self, texture: &Texture, f: F)
    where
        F: FnOnce(&mut RenderPass),
    {
        let mut r = RenderPass {
            operations: Vec::new(),
        };

        f(&mut r);

        let clear_color = { wgpu::Color::BLACK }; // TODO: parse for first clear color operation (if any)

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let view = &texture.view;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        for operation in r.operations {
            match operation {
                RenderOperation::Draw(vertex_array) => {
                    pass.set_pipeline(&vertex_array.pipeline);
                    pass.set_vertex_buffer(0, vertex_array.vertex_buffer.slice(..));

                    if let Some(index) = &vertex_array.index_buffer {
                        pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..vertex_array.index_count, 0, 0..1);
                    } else {
                        pass.draw(0..vertex_array.vertex_count, 0..1);
                    }
                }
                _ => {}
            }
        }

        drop(pass); // drop the mut reference to encoder

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render_window<F>(&self, window: &WindowSurface, f: F)
    where
        F: FnOnce(&mut RenderPass),
    {
        let mut r = RenderPass {
            operations: Vec::new(),
        };

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
        let size = buffer.buffer.size();

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

        encoder.copy_buffer_to_buffer(&buffer.buffer, 0, &staging_buffer, 0, size);

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
    pub fn write_buffer<T>(&self, buffer: &Buffer, data: &[f32]) -> Result<(), &'static str> {
        if !buffer.buffer.usage().contains(wgpu::BufferUsages::COPY_DST) {
            return Err("Buffer must have COPY_DST usage");
        }

        self.queue
            .write_buffer(&buffer.buffer, 0, bytemuck::cast_slice(data));

        Ok(())
    }

    /// Read the texture bytes from a referenced `Texture` object.
    ///
    /// This function is **thread-blocking**, as reading data from the GPU to the CPU is a slow, inefficient process.
    /// Only recommended for compute-use and not render loops or graphics-heavy work.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType};
    ///
    /// let (width, height) = (2, 2);
    ///
    /// let ctx = Context::new();
    /// let tex = ctx.texture(width, height, &[0x32, 0x32, 0x32, 0x32], TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    ///
    /// assert_eq!(vec![0x32, 0x32, 0x32, 0x32], ctx.read_texture(&tex));
    /// ```
    pub fn read_texture(&self, texture: &Texture) -> Vec<u8> {
        let width = texture.config.width;
        let height = texture.config.height;

        let pixel_size = texture.format.bytes_per_pixel();
        let bytes_per_row = pixel_size * width;
        let size = (bytes_per_row * height) as u64;

        // staging buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // encode copy from texture -> buffer
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

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

        let result: Vec<u8> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

        result
    }
}
