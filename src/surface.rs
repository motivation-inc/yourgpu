pub trait Surface {
    fn format(&self) -> wgpu::TextureFormat;
}
