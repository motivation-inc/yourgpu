//! yourgpu - a modern, simple, and fast graphics API for Rust.
//!
//! For examples and references, see the `examples` folder in [yourgpu's repository](https://github.com/motivation-inc/yourgpu).

mod binding;
mod buffer;
mod caching;
mod context;
mod depth_stencil;
mod program;
mod render_pass;
mod texture;
mod vertex_array;
mod window;

pub use {
    binding::BindingBuilder,
    buffer::Buffer,
    context::Context,
    depth_stencil::{Comparison, DepthConfig, StencilConfig, StencilFaceConfig, StencilOperation},
    program::{ComputeProgram, Program},
    render_pass::{RenderCullMode, RenderFrontFaceMode},
    texture::{Texture, TextureDimension, TextureFormat, TextureType},
    vertex_array::{VertexArray, VertexAttributeFormat, VertexLayoutBuilder},
    window::WindowSurface,
};
