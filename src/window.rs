use crate::surface::Surface;

/// A window surface and surface configuration.
pub struct WindowSurface<'a> {
    pub(crate) window_surface: wgpu::Surface<'a>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl<'a> Surface for WindowSurface<'a> {
    fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
}
