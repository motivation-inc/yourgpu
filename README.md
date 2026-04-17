# `yourgpu`

yourgpu is an easy-to-use modern graphics API for Rust. It dramatically simplifies going from code to screen, taking influence from [ModernGL](https://github.com/moderngl/moderngl), whilst using [wgpu](https://wgpu.rs/) as the rendering backend.  

```rust
use image::{ImageBuffer, Rgba};
use yourgpu::{
    BindingBuilder, BufferType, Context, TextureDimension, TextureFormat, TextureType,
    VertexAttributeFormat, VertexLayoutBuilder,
};

fn main() {
    let mut ctx = Context::new();
    let prog = ctx.program(
        r#"
            @vertex
            fn vs(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
                return vec4<f32>(position, 1.0);
            }
        "#,
        Some(
            r#"
            struct Color {
                value: vec4<f32>,
            };

            @group(0) @binding(0)
            var<uniform> u_color: Color;

            @fragment
            fn fs() -> @location(0) vec4<f32> {
                return u_color.value;
            }
        "#,
        ),
        &[BindingBuilder::new(0).uniform("u_color", 0)],
    );
    let tex = ctx.texture(
        (1080, 1080, 1),
        None,
        TextureFormat::Rgba8Unorm,
        TextureType::RenderAttachment,
        TextureDimension::TwoDimensional,
    );
    let vbo = ctx.buffer(
        &[0.0_f32, 0.6, 0.0, -0.6, -0.6, 0.0, 0.6, -0.6, 0.0],
        BufferType::Vertex,
    );
    let color_buffer = ctx.buffer(&[1.0_f32, 0.5, 0.0, 1.0], BufferType::Uniform);
    let vao = ctx.vertex_array(
        &vbo,
        None,
        VertexLayoutBuilder::new().attr(0, VertexAttributeFormat::Float32x3),
    );

    ctx.render_texture(&prog, &tex, None, |r| {
        r.clear(0.0, 0.0, 0.0, 1.0);
        r.set_uniform("u_color", &color_buffer); 

        r.draw(&vao); // draw
    });

    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(1080, 1080, ctx.read_texture(&tex)).unwrap();
    img.save("output.png").unwrap();
}
```

## Features
- Simpler and easier to learn compared to WGPU
- Contexts are headless by default (no screen required)
- WGSL shading language support
- Exposes `wgpu::*` methods for easy integration with WGPU-dependent libraries like `egui_wgpu` 

## Free & Open-Source

yourgpu is 100% free with no drawbacks or limitations. There is no "premium" version; you get the latest and greatest, all licensed under the GPL-3.0.

All source code is public, to anyone. There is no "hidden mechanism" included in this repository; every reference and used factor exists completely and fully.
