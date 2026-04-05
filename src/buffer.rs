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
    CopyDst,
    CopySrc,
}

/// A GPU allocated buffer wrapped over a `wgpu::Buffer` object.
///
/// This struct is best created directly from the `Context::buffer` method, as it provides a higher level API for working with buffers.
pub struct Buffer {
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) length: u32, // number of elements
}
