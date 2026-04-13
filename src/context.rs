use crate::{
    BindingBuilder, BufferType, TextureType,
    buffer::Buffer,
    caching::{BindGroupKey, PipelineKey},
    program::Program,
    render_pass::{RenderOperation, RenderPass},
    surface::Surface,
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayoutBuilder},
    window::WindowSurface,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
    sync::Arc,
};
use winit::window::Window;

fn align_to_256(n: u32) -> u32 {
    let align = 256;
    ((n + align - 1) / align) * align
}

/// A GPU context object, containing a `wgpu::Instance` and `wgpu::Device`.
///
/// This struct implements methods used for GPU operations, acting as a sort of "central hub" for GPU access and usage.
pub struct Context {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pipeline_cache: HashMap<PipelineKey, Rc<wgpu::RenderPipeline>>,
    bind_group_cache: HashMap<BindGroupKey, Rc<wgpu::BindGroup>>,
    next_id: usize,
}

impl<'a> Context {
    /// Constructs a new `Context` object with a primary-preferred backend (Vulkan, Metal, OpenGL, or DX12).
    ///
    /// This function is **thread-blocking**, as gaining access to a GPU context is not instantaneous.
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
            pipeline_cache: HashMap::new(),
            bind_group_cache: HashMap::new(),
            next_id: 0,
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

        surface.configure(&self.device, &config);

        WindowSurface {
            window_surface: surface,
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
    pub fn program(
        &mut self,
        vertex_shader: &str,
        fragment_shader: Option<&str>,
        binding: BindingBuilder,
    ) -> Program {
        let vs_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(vertex_shader.into()),
            });
        let fs_module = fragment_shader.map(|fs| {
            self.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(fs.into()),
                })
        });

        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &binding
                        .entries
                        .values()
                        .map(|b| b.to_owned())
                        .collect::<Vec<wgpu::BindGroupLayoutEntry>>(),
                });
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

        let program = Program {
            id: self.next_id,
            pipeline_layout_id: self.next_id,
            bind_group_layout_id: self.next_id + binding.entries.keys().len(),
            vertex_shader: vs_module,
            fragment_shader: fs_module,
            bind_group_layout,
            pipeline_layout,
            bindings: binding.entries,
        };

        self.next_id += 1;

        program
    }

    /// Constructs a new `Buffer` object, where `data` is an array of data.
    ///
    /// `Buffer` objects store an array of unformatted memory allocated on the GPU, and are used for
    /// GPU-based data storage.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, BufferType};
    ///
    /// let ctx = Context::new();
    /// let buffer = ctx.buffer(&[0.0, 0.0, 0.0], BufferType::Vertex);
    /// ```
    pub fn buffer<T>(&mut self, data: &[T], buffer_type: BufferType) -> Buffer
    where
        T: bytemuck::Pod,
    {
        let bytes = bytemuck::cast_slice(data);
        let byte_size = (std::mem::size_of_val(data)) as u64;

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
                BufferType::Uniform => {
                    wgpu::BufferUsages::UNIFORM
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

        let buffer = Buffer {
            id: self.next_id,
            buffer,
            byte_size,
        };

        self.next_id += 1;

        buffer
    }

    /// Constructs a new `Texture` object, with `width` and `height`, `bytes` being the image data,
    /// and `format` being the texture format of the image.
    ///
    /// If `bytes` is `None`, the texture will be created as an empty texture object.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType};
    ///
    /// let (width, height) = (2, 2);
    ///
    /// let ctx = Context::new();
    /// let tex = ctx.texture(width, height, Some(&[0x32, 0x32, 0x32, 0x32]), TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    /// ```
    pub fn texture(
        &mut self,
        width: u32,
        height: u32,
        bytes: Option<&[u8]>,
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

        if let Some(data) = bytes {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(format.bytes_per_pixel() * width),
                    rows_per_image: Some(height),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let texture = Texture {
            id: self.next_id,
            format,
            texture,
            view,
            sampler,
            width,
            height,
        };

        self.next_id += 1;

        texture
    }

    /// Constructs a new `VertexArray` object, where `vertex_buffer` is a vertex buffer, `index_buffer`
    /// is the optional index buffer, and `vertex_layout` describes the vertex data.
    ///
    /// # Example
    ///
    /// ```
    /// panic!("UNIMPLEMENTED");
    /// ```
    pub fn vertex_array(
        &self,
        vertex_buffer: &Buffer,
        index_buffer: Option<&Buffer>,
        vertex_layout: VertexLayoutBuilder,
    ) -> VertexArray {
        let mut stride = 0;
        let mut attributes = Vec::new();

        for attr in &vertex_layout.attributes {
            attributes.push(wgpu::VertexAttribute {
                format: attr.format.to_wgpu(),
                offset: stride,
                shader_location: attr.location,
            });

            stride += attr.format.size();
        }

        VertexArray {
            stride,
            attributes,
            vertex_buffer: vertex_buffer.buffer.clone(),
            index_buffer: index_buffer.map(|b| b.buffer.clone()),
            vertex_count: (vertex_buffer.byte_size / stride) as u32,
            index_count: index_buffer.map(|b| (b.byte_size / 4) as u32).unwrap_or(0), // index buffer data is a u32
        }
    }

    /// A single render pass for a `Texture` object.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType};
    ///
    /// let ctx = Context::new();
    /// let tex = ctx.texture(
    ///     1028,
    ///     1028,
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    /// );
    /// let program = ctx.program();
    ///
    /// ctx.render_texture(&tex, None |r| {
    ///     r.clear(0.0, 1.0, 0.0, 1.0) // solid green
    /// })
    /// ```
    pub fn render_texture<F>(
        &mut self,
        program: &Program,
        texture: &Texture,
        depth_texture: Option<&Texture>,
        f: F,
    ) where
        F: FnOnce(&mut RenderPass<'a>),
    {
        let mut r = RenderPass {
            operations: Vec::new(),
        };

        f(&mut r);

        let clear_color = r
            .operations
            .iter()
            .find_map(|op| match op {
                RenderOperation::Clear(red, green, blue, alpha) => Some(wgpu::Color {
                    r: *red,
                    g: *green,
                    b: *blue,
                    a: *alpha,
                }),
                _ => None,
            })
            .unwrap_or(wgpu::Color::BLACK);

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
            depth_stencil_attachment: match depth_texture {
                Some(t) => Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // farthest depth
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                None => None,
            },
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // which binding names are actually defined in the program
        let valid_binding_names = program
            .bindings
            .keys()
            .map(|b| b.to_owned())
            .collect::<Vec<String>>();
        let mut buffers: HashMap<String, &'a Buffer> = HashMap::new();
        let mut textures: HashMap<String, &'a Texture> = HashMap::new();
        let mut cull_mode: Option<wgpu::Face> = wgpu::PrimitiveState::default().cull_mode;
        let mut front_face: wgpu::FrontFace = wgpu::PrimitiveState::default().front_face;
        let mut depth_stencil_state: Option<wgpu::DepthStencilState> = None;

        for operation in r.operations {
            match operation {
                RenderOperation::SetViewport(x, y, w, h, min_depth, max_depth) => {
                    pass.set_viewport(x, y, w, h, min_depth, max_depth);
                }
                RenderOperation::SetScissorRect(x, y, w, h) => {
                    pass.set_scissor_rect(x, y, w, h);
                }
                RenderOperation::SetCullMode(mode) => cull_mode = mode,
                RenderOperation::SetFrontFace(face) => front_face = face,
                RenderOperation::SetDepthTest(write, compare) => {
                    if let Some(format) = depth_texture.map(|f| f.format()) {
                        depth_stencil_state = Some(wgpu::DepthStencilState {
                            format,
                            depth_write_enabled: write,
                            depth_compare: compare,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        });
                    }
                }
                RenderOperation::SetUniform(name, buffer) => {
                    if !valid_binding_names.contains(&name) {
                        panic!("Unknown program binding name: '{name}'")
                    }

                    buffers.insert(name, buffer);
                }
                RenderOperation::SetTexture(name, texture) => {
                    if !valid_binding_names.contains(&name) {
                        panic!("Unknown program binding name: '{name}'")
                    }

                    textures.insert(name, texture);
                }
                RenderOperation::Draw(vertex_array) => {
                    let mut buffer_ids: Vec<usize> = Vec::new();
                    let mut texture_ids: Vec<usize> = Vec::new();

                    let entries: Vec<_> = program
                        .bindings
                        .iter()
                        .map(|(name, binding)| match binding.ty {
                            wgpu::BindingType::Buffer {
                                ty: _,
                                has_dynamic_offset: _,
                                min_binding_size: _,
                            } => {
                                let buffer = buffers.get(name).unwrap();
                                buffer_ids.push(buffer.id);

                                wgpu::BindGroupEntry {
                                    binding: binding.binding,
                                    resource: buffer.buffer.as_entire_binding(),
                                }
                            }
                            wgpu::BindingType::Texture {
                                sample_type: _,
                                view_dimension: _,
                                multisampled: _,
                            } => {
                                let tex = textures.get(name).unwrap();
                                texture_ids.push(tex.id);

                                wgpu::BindGroupEntry {
                                    binding: binding.binding,
                                    resource: wgpu::BindingResource::TextureView(&tex.view),
                                }
                            }
                            wgpu::BindingType::Sampler(_) => {
                                let tex = textures.get(name).unwrap();
                                texture_ids.push(tex.id);

                                wgpu::BindGroupEntry {
                                    binding: binding.binding,
                                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                                }
                            }
                            _ => {
                                panic!("Unknown binding type.")
                            }
                        })
                        .collect();

                    buffer_ids.sort();
                    texture_ids.sort();

                    let bind_group = self.get_or_create_bind_group(
                        &program,
                        &buffer_ids,
                        &texture_ids,
                        &entries,
                    );
                    let pipeline = self.get_or_create_pipeline(
                        &program,
                        texture.format(),
                        cull_mode,
                        front_face,
                        depth_stencil_state.to_owned(),
                        &vertex_array,
                    );

                    pass.set_pipeline(&pipeline);
                    pass.set_bind_group(0, &*bind_group, &[]);
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
    pub fn read_buffer(&self, buffer: &Buffer) -> Vec<u8> {
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

        let result: Vec<u8> = bytemuck::cast_slice(&data).to_vec();

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
    /// let tex = ctx.texture(width, height, Some(&[0x32, 0x32, 0x32, 0x32]), TextureFormat::Rgba8Unorm, TextureType::RenderAttachment);
    ///
    /// assert_eq!(vec![0x32, 0x32, 0x32, 0x32], ctx.read_texture(&tex));
    /// ```
    pub fn read_texture(&self, texture: &Texture) -> Vec<u8> {
        let width = texture.width;
        let height = texture.height;

        let bytes_per_row = align_to_256(width * texture.format.bytes_per_pixel());
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

        let padded = bytes_per_row as usize;
        let unpadded = (texture.width * texture.format.bytes_per_pixel()) as usize;

        let mut result = Vec::with_capacity(unpadded * texture.height as usize);

        for row in 0..texture.height as usize {
            let start = row * padded;
            let end = start + unpadded;
            result.extend_from_slice(&data[start..end]);
        }

        drop(data);
        staging_buffer.unmap();

        result
    }

    fn get_or_create_pipeline(
        &mut self,
        program: &Program,
        texture_format: wgpu::TextureFormat,
        cull_mode: Option<wgpu::Face>,
        front_face: wgpu::FrontFace,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
        vertex_array: &VertexArray,
    ) -> Rc<wgpu::RenderPipeline> {
        let mut vertex_hasher = DefaultHasher::new();
        let mut depth_hasher = DefaultHasher::new();
        vertex_array.attributes.hash(&mut vertex_hasher);
        depth_stencil_state.hash(&mut depth_hasher);

        let key = PipelineKey {
            program_id: program.id,
            layout_id: program.pipeline_layout_id,
            attribute_hash: vertex_hasher.finish(),
            depth_stencil_state_hash: depth_hasher.finish(),
        };

        let color_target = Some(wgpu::ColorTargetState {
            format: texture_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        });
        let targets = [color_target];

        self.pipeline_cache
            .entry(key)
            .or_insert_with(|| {
                let pipeline =
                    self.device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: None,
                            layout: Some(&program.pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: &program.vertex_shader,
                                entry_point: None,
                                buffers: &[wgpu::VertexBufferLayout {
                                    array_stride: vertex_array.stride,
                                    step_mode: wgpu::VertexStepMode::Vertex,
                                    attributes: &vertex_array.attributes,
                                }],
                                compilation_options: Default::default(),
                            },
                            fragment: program.fragment_shader.as_ref().map(|fs| {
                                wgpu::FragmentState {
                                    module: fs,
                                    entry_point: None,
                                    targets: &targets,
                                    compilation_options: Default::default(),
                                }
                            }),
                            primitive: wgpu::PrimitiveState {
                                cull_mode,
                                front_face,
                                ..Default::default()
                            },
                            depth_stencil: depth_stencil_state,
                            multisample: Default::default(),
                            multiview_mask: None,
                            cache: None,
                        });

                Rc::new(pipeline)
            })
            .clone()
    }

    fn get_or_create_bind_group(
        &mut self,
        program: &Program,
        buffer_ids: &[usize],
        texture_ids: &[usize],
        entries: &[wgpu::BindGroupEntry],
    ) -> Rc<wgpu::BindGroup> {
        let key = BindGroupKey {
            program_id: program.id,
            layout_id: program.bind_group_layout_id,
            buffer_ids: buffer_ids.to_vec(),
            texture_ids: texture_ids.to_vec(),
        };

        self.bind_group_cache
            .entry(key)
            .or_insert_with(|| {
                let bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &program.bind_group_layout,
                    entries,
                });

                Rc::new(bg)
            })
            .clone()
    }
}
