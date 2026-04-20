use image::{ImageBuffer, Rgba};
use yourgpu::{
    BindingBuilder, Context, TextureDimension, TextureFormat, TextureType, VertexAttributeFormat,
    VertexLayoutBuilder,
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

            struct Brightness {
                value: f32,
            };

            @group(0) @binding(0)
            var<uniform> u_color: Color;

            @group(0) @binding(1)
            var<uniform> u_brightness: Brightness;

            @fragment
            fn fs() -> @location(0) vec4<f32> {
                return vec4<f32>(u_color.value.rgb * u_brightness.value, u_color.value.a);
            }
        "#,
        ),
        &[BindingBuilder::new(0)
            .uniform("u_color", 0)
            .uniform("u_brightness", 1)], // describe how the binding is built
    );
    let tex = ctx.texture(
        (1080, 1080, 1),
        None,
        TextureFormat::Rgba8Unorm,
        TextureType::RenderAttachment,
        TextureDimension::TwoDimensional,
    );

    // vertex buffer
    let vbo = ctx.vertex_buffer(&[0.0, 0.6, 0.0, -0.6, -0.6, 0.0, 0.6, -0.6, 0.0]);

    // uniform buffers
    let color_buffer = ctx.uniform_buffer(&[1.0_f32, 0.5, 0.0, 1.0]); // orange
    let brightness_buffer = ctx.uniform_buffer(&[0.5_f32]); // dim

    // vertex array
    let vao = ctx.vertex_array(
        &vbo,
        None,
        VertexLayoutBuilder::new().attr(0, VertexAttributeFormat::Float32x3),
    );

    // render pass
    ctx.render_texture(&prog, &tex, None, |r| {
        r.clear(0.0, 0.0, 0.0, 1.0); // black background

        // set uniforms
        r.set_buffer("u_color", &color_buffer);
        r.set_buffer("u_brightness", &brightness_buffer);

        r.draw(&vao); // draw
    });

    // save the image
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(1080, 1080, ctx.read_texture(&tex)).unwrap();
    img.save("output.png").unwrap();
}
