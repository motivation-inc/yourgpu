use image::{ImageBuffer, Rgba};
use yourgpu::{
    BindGroupBuilder, BindGroupLayoutBuilder, BufferType, Context, TextureFormat, TextureType,
    VertexAttributeFormat, VertexLayoutBuilder,
};

fn main() {
    let ctx = Context::new();
    let tex = ctx.texture(
        1028,
        1028,
        None,
        TextureFormat::Rgba8UnormSrgb,
        TextureType::RenderAttachment,
    );
    let prog = ctx.program("// ...vertex shader", Some("// ...fragment shader"));
    let bind_group_layout =
        ctx.bind_group_layout(BindGroupLayoutBuilder::new().uniform(binding, visibility));
    let vbo = ctx.buffer(
        &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
        BufferType::Vertex,
    );
    let vao = ctx.vertex_array(
        &tex,
        &prog,
        &vbo,
        None,
        VertexLayoutBuilder::new().attr(0, VertexAttributeFormat::Float32x3),
        &[bind_group_layout],
    );

    ctx.render_texture(&tex, |r| {
        r.clear(0.0, 1.0, 0.0, 1.0);
        r.draw(
            &vao,
            &vec![ctx.bind_group(
                &bind_group_layout,
                BindGroupBuilder::new().uniform(binding, buffer),
            )],
        );
    });

    let img =
        ImageBuffer::<Rgba<u8>, _>::from_raw(1028, 1028, ctx.read_texture(&tex).to_vec()).unwrap();

    img.save("output.png").unwrap();
}
