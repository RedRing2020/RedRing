pub trait RenderStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);

    /// 状態更新（デフォルトは空）
    fn update(&mut self) {}
}
