use image::{ImageBuffer, Rgba};
use yourgpu::{
    Context, TextureDimension, TextureFormat, TextureType, VertexAttributeFormat,
    VertexLayoutBuilder,
};

fn main() {
    let mut ctx = Context::new();
    let prog = ctx.program(
        r#"
            struct VSOut {
                @builtin(position) position: vec4<f32>,
                @location(0) color: vec3<f32>,
            };

            @vertex
            fn vs(
                @location(0) position: vec3<f32>,
                @location(1) color: vec3<f32>,
            ) -> VSOut {
                var out: VSOut;
                out.position = vec4<f32>(position, 1.0);
                out.color = color;
                return out;
            }
        "#,
        Some(
            r#"
            @fragment
            fn fs(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
                return vec4<f32>(color, 1.0);
            }
        "#,
        ),
        &[],
    );
    let tex = ctx.texture(
        (1080, 1080, 1),
        None,
        TextureFormat::Rgba8Unorm,
        TextureType::RenderAttachment,
        TextureDimension::TwoDimensional,
    );

    // vertex buffer
    let vbo = ctx.vertex_buffer(&[
        // position    // color
        0.0, 0.6, 0.0, 1.0, 0.0, 0.0, // top = red
        -0.6, -0.6, 0.0, 0.0, 1.0, 0.0, // left = green
        0.6, -0.6, 0.0, 0.0, 0.0, 1.0, // right = blue
    ]);

    // vertex array
    let vao = ctx.vertex_array(
        &vbo,
        None,
        VertexLayoutBuilder::new()
            .attr(0, VertexAttributeFormat::Float32x3) // position
            .attr(1, VertexAttributeFormat::Float32x3), // color
    );

    // render pass
    ctx.render_texture(&prog, &tex, None, |r| {
        r.clear(0.0, 0.0, 0.0, 1.0); // black background

        r.draw(&vao); // draw
    });

    // save the image
    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(1080, 1080, ctx.read_texture(&tex)).unwrap();
    img.save("output.png").unwrap();
}
