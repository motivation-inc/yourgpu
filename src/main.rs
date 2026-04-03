use yourgpu::{BufferType, Context, TextureFormat, VertexAttributeFormat, VertexLayout};

fn main() {
    let ctx = Context::new();
    let tex = ctx.texture(1028, 1028, &[], TextureFormat::Rgba8Unorm);
    let prog = ctx.program("// ...vertex shader", Some("// ...fragment shader"));
    let vbo = ctx.buffer(
        &[0.0, 1.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0],
        BufferType::Vertex,
    );
    let vao = ctx.vertex_array(
        &prog,
        &vbo,
        VertexLayout::new().attr("in_vert", VertexAttributeFormat::Float32x3),
    );

    ctx.render_texture(&tex, |r| {
        r.clear(0.0, 0.0, 0.0, 0.0);
        r.draw(&prog, &vao);
    });

    // psuedo-code Image::save("file.png", ctx.read_texture(&tex);
}
