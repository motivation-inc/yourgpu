/// A window surface and surface configuration.
///
/// Created using `Context::attach_window`.
pub struct WindowSurface<'a> {
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) config: wgpu::SurfaceConfiguration,
}
