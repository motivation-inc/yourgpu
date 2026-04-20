use yourgpu::{BindingBuilder, Context};

fn main() {
    let mut ctx = Context::new();
    let compute_prog = ctx.compute_program(
        r#"
            @group(0) @binding(0)
            var<storage, read_write> data: array<f32>;

            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                let i = id.x;

                data[i] = data[i] * 2.0;
            }
        "#,
        &[BindingBuilder::new(0).storage("data", 0, false)],
    );

    let input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0]; // data to operate on

    let buf = ctx.storage_buffer(&input);

    // compute pass
    ctx.compute(&compute_prog, |r| {
        r.set_buffer("data", &buf);

        let len = input.len() as u32;
        let workgroup_size = 64;

        let num_groups = (len + workgroup_size - 1) / workgroup_size;

        r.dispatch_workgroups(num_groups, 1, 1);
    });

    let result: Vec<f32> = bytemuck::cast_slice(&ctx.read_buffer(&buf)).to_vec();
    println!("{:?}", result); // the GPU multiplied everything by 2, resulting in an array like [2.0, 4.0, 6.0, 8.0]
}
