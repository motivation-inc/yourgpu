use image::{ImageBuffer, Rgba};
use yourgpu::{
    BufferType, Context, TextureFormat, TextureType, VertexAttributeFormat, VertexLayout,
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
    // let prog = ctx.program("// ...vertex shader", Some("// ...fragment shader"));
    // let vbo = ctx.buffer(
    //     &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
    //     BufferType::Vertex,
    // );
    // let vao = ctx.vertex_array(
    //     &tex,
    //     &prog,
    //     &vbo,
    //     VertexLayout::new().attr("in_vert", VertexAttributeFormat::Float32x3),
    // );

    ctx.render_texture(&tex, |r| {
        r.clear(0.0, 1.0, 0.0, 1.0);
        // r.draw(&prog, &vao);
    });

    let img =
        ImageBuffer::<Rgba<u8>, _>::from_raw(1028, 1028, ctx.read_texture(&tex).to_vec()).unwrap();

    img.save("output.png").unwrap();
}
