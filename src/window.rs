use crate::surface::Surface;

/// A window surface and surface configuration.
pub struct WindowSurface<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}

impl<'a> Surface for WindowSurface<'a> {
    fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
