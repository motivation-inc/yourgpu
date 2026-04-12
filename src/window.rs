use crate::surface::Surface;

/// A window surface and surface configuration.
pub struct WindowSurface<'a> {
    pub(crate) window_surface: wgpu::Surface<'a>,
}

impl<'a> Surface for WindowSurface<'a> {
    fn format(&self) -> wgpu::TextureFormat {
        self.window_surface.get_configuration().unwrap().format
    }
}
