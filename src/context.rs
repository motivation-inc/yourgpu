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

        Program::new(vs_module, fs_module)
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
    pub fn buffer<T>(&self, data: &T, buffer_type: BufferType) -> Buffer
    where
        T: bytemuck::Pod,
    {
        let bytes = bytemuck::bytes_of(data);
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

        Buffer::new(buffer)
    }

    /// Constructs a new `Texture` object, with `width` and `height`, `data` being the image data,
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
    /// let tex = ctx.texture(width, height, &[0.0, 0.0, 0.0, 0.0], TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    /// ```
    pub fn texture<T>(
        &self,
        width: u32,
        height: u32,
        data: &T,
        format: TextureFormat,
        texture_type: TextureType,
    ) -> Texture
    where
        T: bytemuck::Pod,
    {
        let bytes = bytemuck::bytes_of(data);

        if bytes.len() == (width * height) as usize {
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
                    bytes_per_row: Some(4 * width),
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

            Texture::new(texture, view, sampler, config)
        } else {
            panic!("Input data must be same in size as width * height");
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

        for attr in layout.attributes {
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
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let config = surface.config();

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: program.vertex_shader(),
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_buffer_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &program.fragment_shader().unwrap(),
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

        VertexArray::new(
            pipeline,
            vertex_buffer.inner().clone(),
            index_buffer.map(|b| b.inner().clone()),
        )
    }

    pub fn render_texture<F, T>(&self, texture: &Texture, f: F)
    where
        T: bytemuck::Pod,
        F: FnOnce(&mut RenderPass<T>),
    {
        let mut r = RenderPass {
            operations: Vec::new(),
        };

        f(&mut r);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let view = &texture.view;

        for operation in r.operations {
            match operation {
                RenderOperation::Clear(r, g, b, a) => {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                }
                RenderOperation::Draw(program, vertex_array) => {}
                RenderOperation::SetUniform(location, data) => {}
            }
        }

        self.queue.submit(Some(encoder.finish()));
    }

    pub fn render_window<F, T>(&self, window: &WindowSurface, f: F)
    where
        T: bytemuck::Pod,
        F: FnOnce(&mut RenderPass<T>),
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

    /// Read data from a referenced `Texture` object.
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
    /// let tex = ctx.texture(width, height, &[0.0, 0.0, 0.0, 0.0], TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    ///
    /// assert_eq!(vec![0.0, 0.0, 0.0], ctx.read_texture(&tex));
    /// ```
    pub fn read_texture(&self, texture: &Texture) -> &[f32] {
        &[]
    }
}
