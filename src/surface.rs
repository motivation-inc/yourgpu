pub trait Surface {
    fn config(&self) -> &wgpu::SurfaceConfiguration;
}
