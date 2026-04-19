/// A window surface and surface configuration.
///
/// Created using `Context::window_surface`.
pub struct WindowSurface<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl<'a> WindowSurface<'a> {
    /// Resizes the surface configuration.
    ///
    /// - `width`: the width (in pixels)
    /// - `height`: the height (in pixels)
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
        }
    }
}
