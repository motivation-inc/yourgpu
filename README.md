# `yourgpu`

yourgpu is an easy-to-use modern graphics API for Rust. It dramatically simplifies going from code to screen, taking influence from [ModernGL](https://github.com/moderngl/moderngl), whilst using [wgpu](https://wgpu.rs/) as the rendering backend.  

```rust
use yourgpu::Context;

fn main() {
    let mut ctx = Context::new();
    let buf = ctx.storage_buffer(
        b"Hello world!",
    );
    
    println!("Data from GPU: {}", String::from_utf8_lossy(&ctx.read_buffer(&buf)))
}
```

## Features
- Simple and easy to learn :)
- Contexts are headless by default (no screen required)
- First-class [winit](https://github.com/rust-windowing/winit) support
- Built around the [WebGPU Shading Language](https://www.w3.org/TR/WGSL/)
- Exposed `wgpu::*` methods for easy integration with WGPU-dependent libraries like `egui_wgpu` 

## Using `yourgpu`

Everything starts at a `Context` object.

```rust
use yourgpu::Context;

fn main() {
    let mut ctx = Context::new();
}
```

`Context` objects give you access to GPU functions, acting as a baseline for all operations. To create a shader program to run on the GPU, we create a `Program`.

```rust
// ...
let prog = ctx.program("// vertex shader", Some("// fragment shader"), &[]);
```

`Program` objects allow us to also describe shader binding groups, via a `BindGroupBuilder` object. To create data for the GPU, we create a `Buffer`.

```rust
// ...
// A `Buffer` with the vertex type
let vbo = ctx.vertex_buffer(&[    
    0.0,  0.6, 0.0,
   -0.6, -0.6, 0.0,
    0.6, -0.6, 0.0,
]);
```

`Buffer` objects store a variety of data on the GPU, from color to position data. To describe how the vertex buffer is used, we create a `VertexArray`.

```rust
// ...
let vao = ctx.vertex_array(
    &vbo,
    None,
    VertexLayoutBuilder::new()
        .attr(0, VertexAttributeFormat::Float32x3) // position
);
```

`VertexArray` objects describe how to data of a vertex buffer should be used, where `VertexLayoutBuilder` allows a set of attributes and locations describing types. To render the data to a target, like a window, or texture, we create a `RenderPass`.

```rust
// ...
// Example: render to a texture
// Where `tex` is a `Texture` object
ctx.render_texture(&prog, &tex, None,  |r| {
    r.clear(0.0, 0.0, 0.0, 1.0); // black background
    r.draw(&vao); // draw the data
});
```

For the full example, see the file in the [examples folder](https://github.com/motivation-inc/yourgpu/blob/main/examples/01_triangle.rs).

## Free & Open-Source

yourgpu is 100% free with no drawbacks or limitations. There is no "premium" version; you get the latest and greatest, all licensed under the GPL-3.0.

All source code is public, to anyone. There is no "hidden mechanism" included in this repository; every reference and used factor exists completely and fully.
