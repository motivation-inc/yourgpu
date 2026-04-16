/// A window surface and surface configuration.
///
/// Created using `Context::attach_window`.
pub struct WindowSurface<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl<'a> WindowSurface<'a> {
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
        }
    }
}
