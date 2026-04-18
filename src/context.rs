use crate::{
    BindingBuilder, DepthConfig, StencilConfig, TextureDimension, TextureType,
    buffer::{Buffer, BufferType},
    caching::{BindGroupKey, PipelineKey},
    program::Program,
    render_pass::{RenderOperation, RenderPass},
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayoutBuilder},
    window::WindowSurface,
};
use std::{
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

/// A GPU context object.
///
/// This struct implements methods used for GPU operations, acting as a sort of "central hub" for GPU
/// access and usage.
pub struct Context {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline_cache: HashMap<PipelineKey, Rc<wgpu::RenderPipeline>>,
    bind_group_cache: HashMap<BindGroupKey, Rc<HashMap<u32, wgpu::BindGroup>>>,
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
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .expect("Unable to find an srgb surface format for window surface");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&self.device, &config); // IMPORTANT: configure surface to the device

        WindowSurface { surface, config }
    }

    /// Constructs a new `Program` object,
    ///
    /// - `vertex_shader`: the vertex shader source
    /// - `fragment_shader`: the fragment shader source (if `None`, the program defaults to no fragment shader)
    /// - `bindings`: an array of `BindingBuilder`, where each `BindingBuilder` in `bindings` represents a single binding group
    ///
    /// This function requires the shader source to be valid [WebGPU Shading Language](https://www.w3.org/TR/WGSL/).
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, BindingBuilder};
    ///
    /// let mut ctx = Context::new();
    /// let prog = ctx.program("// vertex shader", Some("// fragment shader"), &[BindingBuilder::new(0)]);
    /// ```
    pub fn program(
        &mut self,
        vertex_shader: &str,
        fragment_shader: Option<&str>,
        bindings: &[BindingBuilder],
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

        let mut bind_group_entries = HashMap::new();
        let mut bind_group_layouts = HashMap::new();
        let mut entry_names = Vec::new();

        for binding in bindings {
            for (name, _) in &binding.entries {
                entry_names.push(name.to_owned());
            }

            bind_group_entries.insert(binding.group, binding.entries.clone());
            bind_group_layouts.insert(
                binding.group,
                self.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: None,
                        entries: &binding
                            .entries
                            .values()
                            .map(|b| b.to_owned())
                            .collect::<Vec<wgpu::BindGroupLayoutEntry>>(),
                    }),
            );
        }

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layouts
                    .values()
                    .collect::<Vec<&wgpu::BindGroupLayout>>(),
                immediate_size: 0,
            });

        let program = Program {
            id: self.next_id,
            pipeline_layout_id: self.next_id,
            bind_group_layout_id: self.next_id,
            vertex_shader: vs_module,
            fragment_shader: fs_module,
            bind_group_layouts,
            pipeline_layout,
            bind_group_entries,
            entry_names,
        };

        self.next_id += 1;

        program
    }

    /// Constructs a new `Buffer` object with the vertex usage.
    ///
    /// - `data`: an array of f32 vertex data
    ///
    /// Vertex buffers are read by the vertex shader using the configured vertex layout (implicated by
    /// the `VertexArray` object.)
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let mut ctx = Context::new();
    /// let buf = ctx.vertex_buffer(
    ///     &[
    ///         0.0, 0.6, 0.0,
    ///        -0.6, -0.6, 0.0,
    ///         0.6, -0.6, 0.0
    ///     ]
    /// );
    /// ```
    pub fn vertex_buffer(&mut self, data: &[f32]) -> Buffer {
        self.buffer(data, BufferType::Vertex)
    }

    /// Constructs a new `Buffer` object with the index usage.
    ///
    /// - `data`: an array of u32 index data
    ///
    /// Index buffers define how vertices are reused during drawing.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let mut ctx = Context::new();
    /// let buf = ctx.index_buffer(&[0, 1, 2]); // triangle
    /// ```
    pub fn index_buffer(&mut self, data: &[u32]) -> Buffer {
        self.buffer(data, BufferType::Index)
    }

    /// Constructs a new `Buffer` object with the uniform usage.
    ///
    /// - `data`: uniform data
    ///
    /// Uniform buffers are read-only inputs shared across shader invocations. Data must
    /// follow [WGSL alignment rules](https://webgpufundamentals.org/webgpu/lessons/webgpu-memory-layout.html).
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    /// use bytemuck::{Pod, Zeroable};
    ///
    /// #[derive(Copy, Clone, Pod, Zeroable)]
    /// #[repr(C)]
    /// struct Color {
    ///     pub r: f32,
    ///     pub g: f32,
    ///     pub b: f32,
    ///     pub a: f32,
    /// }
    ///
    /// let mut ctx = Context::new();
    /// let buf = ctx.uniform_buffer(
    ///     &Color {
    ///         r: 0.0,
    ///         g: 1.0,
    ///         b: 0.0,
    ///         a: 1.0,
    ///     }
    /// ); // color buffer
    /// ```
    pub fn uniform_buffer<T: bytemuck::Pod>(&mut self, data: &T) -> Buffer {
        self.buffer(std::slice::from_ref(data), BufferType::Uniform)
    }

    /// Constructs a new `Buffer` object with the storage usage.
    ///
    /// - `data`: storage data
    ///
    /// Storage buffers only allow read-write data access in shaders.
    pub fn storage_buffer<T: bytemuck::Pod>(&mut self, data: &[T]) -> Buffer {
        self.buffer(data, BufferType::Storage)
    }

    fn buffer<T: bytemuck::Pod>(&mut self, data: &[T], buffer_type: BufferType) -> Buffer {
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
                BufferType::Storage => wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                BufferType::Uniform => {
                    wgpu::BufferUsages::UNIFORM
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC
                }
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

    /// Constructs a new `Texture` object.
    ///
    /// - `size`: the width, height, and depth (values > 1 make this a 3D texture)
    /// - `bytes`: an array of texture bytes (if `None`, the texture will be initialized as an empty buffer)
    /// - `format`: the texture format,
    /// - `texture_type`: the texture type
    /// - `dimension`: being the texture's view dimension.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType, TextureDimension};
    ///
    /// let (width, height, depth) = (2, 2, 1);
    ///
    /// let mut ctx = Context::new();
    /// let tex = ctx.texture(
    ///     (width, height, depth),
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    ///     TextureDimension::TwoDimensional
    /// ); // empty texture
    /// ```
    pub fn texture(
        &mut self,
        size: (u32, u32, u32),
        bytes: Option<&[u8]>,
        format: TextureFormat,
        texture_type: TextureType,
        dimension: TextureDimension,
    ) -> Texture {
        let (width, height, depth) = size;

        if (width > 8192 || height > 8192) || (width > 8192 && height > 8192) {
            panic!("Width or heigth of the texture exceeds the limit of 8192")
        }

        let extent = wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: depth,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: match dimension {
                TextureDimension::TwoDimensional | TextureDimension::TwoDimensionalArray => {
                    wgpu::TextureDimension::D2
                }
                _ => wgpu::TextureDimension::D3,
            },
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
                extent,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(format.to_wgpu()),
            aspect: wgpu::TextureAspect::All,
            dimension: Some(dimension.to_wgpu()),
            ..Default::default()
        });
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

    /// Constructs a new `VertexArray` object.
    ///
    /// - `vertex_buffer`: the vertex buffer
    /// - `index_buffer`: the index buffer (if `None`, the index buffer is left uninitialized)
    /// - `vertex_layout`: how the vertex data is packed
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, VertexLayoutBuilder, VertexAttributeFormat, BindingBuilder};
    ///
    /// let mut ctx = Context::new();
    /// let prog = ctx.program("// vertex_shader", Some("// fragment_shader"), &[BindingBuilder::new(0)]);
    /// let vbo = ctx.vertex_buffer(
    ///     &[
    ///         0.0, 0.6, 0.0,
    ///         -0.6, -0.6, 0.0,
    ///         0.6, -0.6, 0.0
    ///     ],
    /// );
    ///
    /// let vao = ctx.vertex_array(
    ///     &vbo,
    ///     None,
    ///     VertexLayoutBuilder::new().attr(0, VertexAttributeFormat::Float32x3)
    /// );
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

        let vertex_count = if stride == 0 {
            0
        } else {
            (vertex_buffer.byte_size / stride) as u32
        };

        VertexArray {
            stride,
            attributes,
            vertex_buffer: vertex_buffer.buffer.clone(),
            index_buffer: index_buffer.map(|b| b.buffer.clone()),
            vertex_count: vertex_count,
            index_count: index_buffer.map(|b| (b.byte_size / 4) as u32).unwrap_or(0), // index buffer data is a u32
        }
    }

    /// A single render pass for a `Texture` object.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType, TextureDimension, BindingBuilder};
    ///
    /// let mut ctx = Context::new();
    /// let tex = ctx.texture(
    ///     (1028, 1028, 1),
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    ///     TextureDimension::TwoDimensional
    /// );
    /// let prog = ctx.program("// vertex shader", Some("// fragment shader"), &[BindingBuilder::new(0)]);
    ///
    /// ctx.render_texture(&prog, &tex, None, |r| {
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
            clear: wgpu::Color::BLACK,
            operations: Vec::new(),
        };

        f(&mut r);

        self.render_view(
            program,
            &texture.view,
            texture.format.to_wgpu(),
            depth_texture.map(|d| &d.view),
            r.clear,
            r.operations,
        );
    }

    /// A single render pass for a `WindowSurface` object.
    ///
    /// This function will request the current frame from the window, potentially panicking if the
    /// function fails to acquire the next swap chain texture.
    pub fn render_window<F>(&mut self, program: &Program, window: &WindowSurface, f: F)
    where
        F: FnOnce(&mut RenderPass<'a>),
    {
        let mut r = RenderPass {
            clear: wgpu::Color::BLACK,
            operations: Vec::new(),
        };

        f(&mut r);

        let frame = match window.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                // reconfigure the surface with the existing config to sync with the window
                window.surface.configure(&self.device, &window.config);

                return;
            }
            // battery dead, screen likely shut
            Err(wgpu::SurfaceError::Lost) => {
                window.surface.configure(&self.device, &window.config);

                return;
            }
            Err(e) => panic!("Persistent Surface Error: {:?}", e),
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_view(
            program,
            &view,
            window.config.format,
            None,
            r.clear,
            r.operations,
        );

        frame.present();
    }

    fn render_view(
        &mut self,
        program: &Program,
        view: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        depth_view: Option<&wgpu::TextureView>,
        clear_color: wgpu::Color,
        operations: Vec<RenderOperation<'a>>,
    ) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
            depth_stencil_attachment: depth_view.map(|dv| wgpu::RenderPassDepthStencilAttachment {
                view: dv,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let mut buffers: HashMap<String, &'a Buffer> = HashMap::new();
        let mut textures: HashMap<String, &'a Texture> = HashMap::new();
        let mut cull_mode: Option<wgpu::Face> = wgpu::PrimitiveState::default().cull_mode;
        let mut front_face: wgpu::FrontFace = wgpu::PrimitiveState::default().front_face;
        let mut depth_config: Option<DepthConfig> = None;
        let mut stencil_config: Option<StencilConfig> = None;

        for operation in operations {
            match operation {
                RenderOperation::SetViewport(x, y, w, h, min_depth, max_depth) => {
                    pass.set_viewport(x, y, w, h, min_depth, max_depth);
                }
                RenderOperation::SetScissorRect(x, y, w, h) => {
                    pass.set_scissor_rect(x, y, w, h);
                }
                RenderOperation::SetCullMode(mode) => cull_mode = mode,
                RenderOperation::SetFrontFace(face) => front_face = face,
                RenderOperation::SetDepthConfig(cfg) => depth_config = cfg,
                RenderOperation::SetStencilConfig(cfg) => stencil_config = cfg,
                RenderOperation::SetStencilReference(rf) => {
                    pass.set_stencil_reference(rf);
                }
                RenderOperation::SetUniform(name, buffer) => {
                    if !program.entry_names.contains(&name) {
                        panic!("Unknown program binding name: '{name}'")
                    }

                    buffers.insert(name, buffer);
                }
                RenderOperation::SetTexture(name, texture) => {
                    if !program.entry_names.contains(&name) {
                        panic!("Unknown program binding name: '{name}'")
                    }

                    textures.insert(name, texture);
                }
                RenderOperation::Draw(vertex_array) => {
                    let mut buffer_ids: Vec<usize> = Vec::new();
                    let mut texture_ids: Vec<usize> = Vec::new();
                    let mut group_entries: HashMap<u32, Vec<wgpu::BindGroupEntry>> = HashMap::new();

                    for (group, bindings) in &program.bind_group_entries {
                        for (name, binding) in bindings {
                            let entry = match binding.ty {
                                wgpu::BindingType::Buffer { .. } => {
                                    let buffer = buffers.get(name).unwrap();
                                    buffer_ids.push(buffer.id);

                                    wgpu::BindGroupEntry {
                                        binding: binding.binding,
                                        resource: buffer.buffer.as_entire_binding(),
                                    }
                                }
                                wgpu::BindingType::Texture { .. } => {
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
                                _ => panic!("Unknown binding type."),
                            };

                            group_entries.entry(*group).or_default().push(entry);
                        }
                    }

                    buffer_ids.sort();
                    texture_ids.sort();

                    let bind_groups = self.get_or_create_bind_group(
                        &program,
                        &buffer_ids,
                        &texture_ids,
                        &group_entries,
                    );
                    let pipeline = self.get_or_create_pipeline(
                        &program,
                        format,
                        depth_view.map(|d| d.texture().format()),
                        cull_mode,
                        front_face,
                        depth_config,
                        stencil_config,
                        &vertex_array,
                    );

                    pass.set_pipeline(&pipeline);

                    for (group, bg) in &*bind_groups {
                        pass.set_bind_group(group.to_owned(), bg, &[]);
                    }

                    pass.set_vertex_buffer(0, vertex_array.vertex_buffer.slice(..));

                    if let Some(index) = &vertex_array.index_buffer {
                        pass.set_index_buffer(index.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..vertex_array.index_count, 0, 0..1);
                    } else {
                        pass.draw(0..vertex_array.vertex_count, 0..1);
                    }
                }
            }
        }

        drop(pass); // drop the mut reference to encoder
        self.queue.submit(Some(encoder.finish()));
    }

    /// Read data (in bytes) from a referenced `Buffer` object.
    ///
    /// - `buffer`: the buffer to read
    ///
    /// This function is **thread-blocking**, as reading data from the GPU to the CPU is a slow,
    /// inefficient process. It is advised to only use this function when reading is strictly necessary.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let mut ctx = Context::new();
    /// let buffer = ctx.vertex_buffer(&[0.0, 0.0, 0.0]);
    ///
    /// let data: Vec<f32> = bytemuck::cast_slice(&ctx.read_buffer(&buffer)).to_vec();
    /// assert_eq!(vec![0.0, 0.0, 0.0], data);
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

    /// Write data to a referenced `Buffer` object.
    ///
    /// - `buffer`: the buffer to write
    /// - `data`: an array of data implementing the `bytemuck::Pod` trait
    ///
    /// # Errors
    ///
    /// This function will return `Err` if the `Buffer` object does not contain the `BufferType::CopyDst` usage.
    /// By default, vertex, index, and storage buffers contain `BufferType::CopyDst`.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let mut ctx = Context::new();
    /// let buffer = ctx.vertex_buffer(&[0.0, 0.0, 0.0]);
    ///
    /// ctx.write_buffer(&buffer, &[1.0_f32, 1.0, 1.0]); // data must be explicitly defined
    /// ```
    pub fn write_buffer<T: bytemuck::Pod>(
        &self,
        buffer: &Buffer,
        data: &T,
    ) -> Result<(), &'static str> {
        if !buffer.buffer.usage().contains(wgpu::BufferUsages::COPY_DST) {
            return Err("Buffer must have COPY_DST usage");
        }

        self.queue.write_buffer(
            &buffer.buffer,
            0,
            bytemuck::cast_slice(std::slice::from_ref(data)),
        );

        Ok(())
    }

    /// Read the texture bytes from a referenced `Texture` object.
    ///
    /// - `texture`: the texture object to read bytes from
    ///
    /// This function is **thread-blocking**, as reading data from the GPU to the CPU is a slow, inefficient process.
    /// Only recommended for compute-use and not render loops or graphics-heavy work.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType, TextureDimension};
    ///
    /// let (width, height, depth) = (2, 2, 1);
    ///
    /// let mut ctx = Context::new();
    /// let tex = ctx.texture(
    ///     (width, height, depth),
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    ///     TextureDimension::TwoDimensional
    /// );
    ///
    /// let data: Vec<i32> = bytemuck::cast_slice(&ctx.read_texture(&tex)).to_vec();
    /// assert_eq!(vec![0, 0, 0, 0], data);
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

    /// Write bytes to a referenced `Texture` object.
    ///
    /// - `texture`: the texture object to read bytes from
    /// - `data`: an array of texture bytes
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::{Context, TextureFormat, TextureType, TextureDimension};
    ///
    /// let (width, height, depth) = (2, 2, 1);
    ///
    /// let mut ctx = Context::new();
    /// let tex = ctx.texture(
    ///     (width, height, depth),
    ///     None,
    ///     TextureFormat::Rgba8Unorm,
    ///     TextureType::RenderAttachment,
    ///     TextureDimension::TwoDimensional
    /// );
    ///
    /// ctx.write_texture(&tex, bytemuck::cast_slice(&[0, 0, 0, 0])); // write all zeros
    /// ```
    pub fn write_texture(&self, texture: &Texture, data: &[u8]) {
        let (width, height) = (texture.width, texture.height);

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture.format.bytes_per_pixel() * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Returns the context's `wgpu::Instance`.
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Returns the context's `wgpu::Adapter`.
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Returns the context's `wgpu::Device`.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Returns the context's `wgpu::Queue`.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn get_or_create_pipeline(
        &mut self,
        program: &Program,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        cull_mode: Option<wgpu::Face>,
        front_face: wgpu::FrontFace,
        depth_config: Option<DepthConfig>,
        stencil_config: Option<StencilConfig>,
        vertex_array: &VertexArray,
    ) -> Rc<wgpu::RenderPipeline> {
        let depth_stencil_state = {
            let dcfg = match depth_config {
                Some(cfg) => Some(cfg),
                None => None,
            };

            if let Some(cfg) = dcfg {
                Some(wgpu::DepthStencilState {
                    format: if let Some(fmt) = depth_format {
                        fmt
                    } else {
                        color_format
                    },
                    depth_write_enabled: cfg.write,
                    depth_compare: cfg.compare,
                    stencil: match stencil_config {
                        Some(stencil) => stencil.to_wgpu(),
                        None => wgpu::StencilState::default(),
                    },
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            }
        };
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
            format: color_format,
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
        group_entries: &HashMap<u32, Vec<wgpu::BindGroupEntry>>,
    ) -> Rc<HashMap<u32, wgpu::BindGroup>> {
        let key = BindGroupKey {
            program_id: program.id,
            layout_id: program.bind_group_layout_id,
            buffer_ids: buffer_ids.to_vec(),
            texture_ids: texture_ids.to_vec(),
        };

        self.bind_group_cache
            .entry(key)
            .or_insert_with(|| {
                let mut bind_groups = HashMap::new();

                for (group, layout) in &program.bind_group_layouts {
                    let entries = group_entries
                        .get(group)
                        .expect("Missing bind group entries for group");

                    bind_groups.insert(
                        group.to_owned(),
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: None,
                            layout: &layout,
                            entries: entries,
                        }),
                    );
                }

                Rc::new(bind_groups)
            })
            .clone()
    }
}
