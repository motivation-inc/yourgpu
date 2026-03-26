use std::collections::HashMap;

pub enum VertexAttributeFormat {
    // floats
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,

    // unsigned ints
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,

    // signed ints
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,

    // normalized
    Unorm8x2,
    Unorm8x4,
    Snorm8x2,
    Snorm8x4,

    // optional
    Float16x2,
    Float16x4,
}

pub struct VertexLayout {
    attributes: HashMap<String, VertexAttributeFormat>,
}

impl VertexLayout {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }
    pub fn attr(mut self, name: &str, format: VertexAttributeFormat) -> Self {
        self.attributes.insert(name.to_string(), format);

        self
    }
}

pub struct VertexArray {}
