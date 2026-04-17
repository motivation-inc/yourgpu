/// Describes the type of buffer.
pub enum BufferType {
    Vertex,
    Index,
    Storage,
    Uniform,
}

/// A buffer storing an array of GPU allocated memory.
///
/// Created using `Context::*_buffer` methods.
pub struct Buffer {
    pub(crate) id: usize,
    pub(crate) buffer: wgpu::Buffer,
    pub(crate) byte_size: u64,
}
