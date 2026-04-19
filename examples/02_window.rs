use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use yourgpu::{
    Buffer, Context, Program, VertexArray, VertexAttributeFormat, VertexLayoutBuilder,
    WindowSurface,
};

struct App {
    ctx: Context,
    window: Option<Arc<Window>>,
    surface: Option<WindowSurface<'static>>,
    prog: Option<Program>,
    vao: Option<VertexArray>,
    vbo: Option<Buffer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attrs = Window::default_attributes().with_title("yourgpu Window");
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        self.window = Some(window.clone());
        self.surface = Some(self.ctx.window_surface(window.clone()));

        // initialize resources once
        let prog = self.ctx.program(
            r#"
                struct VSOut {
                    @builtin(position) position: vec4<f32>,
                    @location(0) color: vec3<f32>,
                };
                @vertex
                fn vs(@location(0) pos: vec3<f32>, @location(1) col: vec3<f32>) -> VSOut {
                    var out: VSOut;
                    out.position = vec4<f32>(pos, 1.0);
                    out.color = col;
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

        let vbo = self.ctx.vertex_buffer(&[
            0.0, 0.6, 0.0, 1.0, 0.0, 0.0, -0.6, -0.6, 0.0, 0.0, 1.0, 0.0, 0.6, -0.6, 0.0, 0.0, 0.0,
            1.0,
        ]);

        let vao = self.ctx.vertex_array(
            &vbo,
            None,
            VertexLayoutBuilder::new()
                .attr(0, VertexAttributeFormat::Float32x3)
                .attr(1, VertexAttributeFormat::Float32x3),
        );

        self.prog = Some(prog);
        self.vbo = Some(vbo);
        self.vao = Some(vao);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut surface) = self.surface {
                    // without this, the surface wouldn't resize to match the window
                    surface.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().expect("Window not initialized");
                let surface = self.surface.as_mut().expect("Surface not initialized");

                // get references to our cached resources
                let prog = self.prog.as_ref().expect("Program not cached");
                let vao = self.vao.as_ref().expect("VAO not cached");

                // final render
                self.ctx.render_window(prog, surface, |r| {
                    r.clear(0.1, 0.1, 0.1, 1.0);
                    r.draw(vao);
                });

                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        ctx: Context::new(),
        window: None,
        surface: None,
        prog: None,
        vbo: None,
        vao: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
