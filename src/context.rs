use crate::{
    buffer::Buffer,
    program::Program,
    render_context::RenderContext,
    texture::{Texture, TextureFormat},
    vertex_array::{VertexArray, VertexLayout},
    window_id::WindowId,
};

pub struct Context {}

impl Context {
    /// Constructs a new context object.
    ///
    /// # Example
    ///
    /// ```
    /// use yourgpu::Context;
    ///
    /// let ctx = Context::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Attaches a [winit `Window` object](https://docs.rs/winit-gtk/latest/winit/window/struct.Window.html) to the context, returning a `WindowId` object referencing the window.
    pub fn attach_window(&self) -> WindowId {
        WindowId(0)
    }

    pub fn program(&self, vertex_shader: &str, fragment_shader: Option<&str>) -> Program {
        Program {}
    }

    pub fn buffer(&self, data: &[f32]) -> Buffer {
        Buffer {}
    }

    pub fn texture(
        &self,
        width: usize,
        height: usize,
        data: &[f32],
        format: TextureFormat,
    ) -> Texture {
        Texture {}
    }

    pub fn vertex_array(
        &self,
        program: &Program,
        buffer: &Buffer,
        layout: VertexLayout,
    ) -> VertexArray {
        VertexArray {}
    }

    pub fn render_texture<F>(&self, texture: &Texture, f: F)
    where
        F: FnOnce(&mut RenderContext),
    {
        let mut r = RenderContext {};

        f(&mut r);
    }

    pub fn render_window<F>(&self, window_id: &WindowId, f: F)
    where
        F: FnOnce(&mut RenderContext),
    {
        let mut r = RenderContext {};

        f(&mut r);
    }

    pub fn read_texture(&self, texture: &Texture) -> &[f32] {
        &[]
    }
}
