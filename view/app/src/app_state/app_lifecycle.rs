//! AppState の画面ライフサイクル処理（主にリサイズ）を扱うモジュール。

use super::AppState;

impl AppState {
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.graphic.config.width = size.width;
        self.graphic.config.height = size.height;
        self.graphic
            .surface
            .configure(&self.graphic.device, &self.graphic.config);

        self.graphic.depth_texture = self
            .graphic
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth Texture (Resized)"),
                size: wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
        self.graphic.depth_view = self
            .graphic
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.update_camera_uniforms();
    }
}
