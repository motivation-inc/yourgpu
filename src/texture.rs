pub enum TextureFormat {
    // color
    Rgba8Unorm,
    Rgba8UnormSrgb,
    R8Unorm,
    Rg8Unorm,

    // float
    Rgba16Float,
    Rgba32Float,

    // depth
    Depth24Plus,
    Depth32Float,

    // integer
    Rgba8Uint,
    Rgba8Sint,
}

pub struct Texture {}
