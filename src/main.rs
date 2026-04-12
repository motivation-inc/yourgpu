use image::{ImageBuffer, Rgba};
use yourgpu::{
    BindingBuilder, BufferType, Context, TextureFormat, TextureType, VertexAttributeFormat,
    VertexLayoutBuilder,
};

fn main() {
    let mut ctx = Context::new();

    let tex = ctx.texture(
        1920,
        1920,
        None,
        TextureFormat::Rgba8Unorm,
        TextureType::RenderAttachment,
    );

    // Vertex shader (pass through)
    let vertex_shader = r#"
        @vertex
        fn vs(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
            return vec4<f32>(position, 1.0);
        }
    "#;

    // Fragment shader reads a uniform color
    let fragment_shader = r#"
        struct Color {
            value: vec4<f32>,
        };

        @group(0) @binding(0)
        var<uniform> u_color: Color;

        @fragment
        fn fs() -> @location(0) vec4<f32> {
            return u_color.value;
        }
    "#;

    let prog = ctx.program(
        vertex_shader,
        Some(fragment_shader),
        BindingBuilder::new().uniform("u_color", 0),
    );

    // Create a uniform buffer (RGBA = orange-ish)
    let color_buffer = ctx.buffer(&[1.0_f32, 0.5, 0.0, 1.0], BufferType::Uniform);

    let vbo = ctx.buffer(
        &[0.0_f32, 0.6, 0.0, -0.6, -0.6, 0.0, 0.6, -0.6, 0.0],
        BufferType::Vertex,
    );

    let vao = ctx.vertex_array(
        &vbo,
        None,
        VertexLayoutBuilder::new().attr(0, VertexAttributeFormat::Float32x3),
    );

    ctx.render_texture(&prog, &tex, |r| {
        r.clear(0.0, 0.0, 0.0, 1.0);
        r.set_uniform("u_color", &color_buffer);

        r.draw(&vao);
    });

    let img =
        ImageBuffer::<Rgba<u8>, _>::from_raw(1920, 1920, ctx.read_texture(&tex).to_vec()).unwrap();

    img.save("output.png").unwrap();
}
