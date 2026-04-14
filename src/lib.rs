//! yourgpu, a modern, simple, and fast graphics API for Rust.
//!
//! For examples and references, see the `examples` folder in [yourgpu's repository](https://github.com/motivation-inc/yourgpu).

mod binding;
mod buffer;
mod caching;
mod context;
mod program;
mod render_pass;
mod surface;
mod texture;
mod vertex_array;
mod window;

pub use {
    binding::BindingBuilder,
    buffer::BufferType,
    context::Context,
    render_pass::{RenderCullMode, RenderDepthComparison, RenderFrontFaceMode},
    texture::{TextureDimension, TextureFormat, TextureType},
    vertex_array::{VertexAttributeFormat, VertexLayoutBuilder},
};
