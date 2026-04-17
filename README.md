# `yourgpu`

yourgpu is an easy-to-use modern graphics API for Rust. It dramatically simplifies going from code to screen, taking influence from [ModernGL](https://github.com/moderngl/moderngl), whilst using [wgpu](https://wgpu.rs/) as the rendering backend.  

```rust
use image::{ImageBuffer, Rgba};
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
- Simpler and easier to learn compared to WGPU
- Contexts are headless by default (no screen required)
- WGSL shading language support
- Exposes `wgpu::*` methods for easy integration with WGPU-dependent libraries like `egui_wgpu` 

## Free & Open-Source

yourgpu is 100% free with no drawbacks or limitations. There is no "premium" version; you get the latest and greatest, all licensed under the GPL-3.0.

All source code is public, to anyone. There is no "hidden mechanism" included in this repository; every reference and used factor exists completely and fully.
