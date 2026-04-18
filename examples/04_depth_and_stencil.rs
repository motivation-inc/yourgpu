use image::{ImageBuffer, Rgba};
use yourgpu::{
    Comparison, Context, DepthConfig, StencilConfig, StencilFaceConfig, StencilOperation,
    TextureDimension, TextureFormat, TextureType, VertexAttributeFormat, VertexLayoutBuilder,
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
            fn vs(@location(0) pos: vec3<f32>, @location(1) color: vec3<f32>) -> VSOut {
                var out: VSOut;
                // Note: we use the z-axis for the depth test
                out.position = vec4<f32>(pos, 1.0);
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
    let depth_tex = ctx.texture(
        (1080, 1080, 1),
        None,
        TextureFormat::Depth24PlusStencil8, // use a depth/stencil format
        TextureType::RenderAttachment,
        TextureDimension::TwoDimensional,
    );

    // setup geometry
    let vbo = ctx.vertex_buffer(&[
        // x, y, z, r, g, b
        0.0, 0.6, 0.5, 1.0, 0.0, 0.0, // triangle at z=0.5
        -0.6, -0.6, 0.5, 0.0, 1.0, 0.0, 0.6, -0.6, 0.5, 0.0, 0.0, 1.0,
    ]);

    let vao = ctx.vertex_array(
        &vbo,
        None,
        VertexLayoutBuilder::new()
            .attr(0, VertexAttributeFormat::Float32x3)
            .attr(1, VertexAttributeFormat::Float32x3),
    );

    // define depth configuration
    // we want to write to the depth buffer and pass if the new fragment is "closer" (Less)
    let depth_state = DepthConfig::new(true, Comparison::Less);

    // define stencil configuration
    // if the test passes, REPLACE the stencil value with our reference
    let stencil_face = StencilFaceConfig::new(
        Comparison::Always,
        StencilOperation::Keep,
        StencilOperation::Keep,
        StencilOperation::Replace,
    );

    let stencil_state = StencilConfig::new(
        stencil_face,
        stencil_face, // same for back
        0xFF,         // read mask
        0xFF,         // write mask
    );

    // render pass
    ctx.render_texture(&prog, &tex, Some(&depth_tex), |r| {
        r.clear(0.1, 0.1, 0.1, 1.0);

        // STEP A: set up the stencil mask
        r.set_stencil_config(Some(stencil_state));
        r.set_stencil_reference(1); // we are marking the area with '1'

        // STEP B: set up depth
        r.set_depth_config(Some(depth_state));

        // draw the triangle
        // this will write '1' into the stencil buffer and '0.5' into depth where the triangle is
        r.draw(&vao);
    });

    let img = ImageBuffer::<Rgba<u8>, _>::from_raw(1080, 1080, ctx.read_texture(&tex)).unwrap();
    img.save("output.png").unwrap();
}
