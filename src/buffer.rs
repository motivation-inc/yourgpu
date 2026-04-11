/// Describes the type of buffer to use.
///
/// `Vertex`: Vertex buffer
/// `Index`: Index buffer
/// `Storage`: Storage buffer
/// `CopySrc`: A source buffer for destination buffers
/// `CopyDst`: A destination buffer for buffer copies
pub enum BufferType {
    Vertex,
    Index,
    Storage,
    Uniform,
    CopyDst,
    CopySrc,
}

/// A GPU allocated buffer.
///
/// Created using `Context::buffer`.
pub struct Buffer {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) element_count: u32,
    pub(crate) byte_size: u64,
}
