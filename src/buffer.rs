/// Describes the type of buffer.
pub enum BufferType {
    /// A vertex buffer.
    Vertex,
    /// An index buffer.
    Index,
    /// A storage buffer.
    Storage,
    /// A uniform buffer.
    Uniform,
    /// A destination buffer for buffer copies.
    CopyDst,
    /// A source buffer for destination buffers.
    CopySrc,
}

/// A GPU allocated buffer.
///
/// Created using `Context::buffer`.
pub struct Buffer {
    pub(crate) id: usize,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) byte_size: u64,
}
