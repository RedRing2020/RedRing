use std::any::Any;

pub trait RenderStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);

    /// 深度バッファ付きレンダリング（デフォルト: depth を無視）
    fn render_with_depth(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _depth_view: &wgpu::TextureView,
    ) {
        // デフォルトでは depth なしで描画
        self.render(encoder, view);
    }

    /// カメラ行列を更新（デフォルトは何もしない）
    ///
    /// ステージが GPU ユニフォームとしてカメラ行列を必要とする場合は、
    /// このメソッドをオーバーライドして実装してください。
    fn update_camera(
        &mut self,
        _queue: &wgpu::Queue,
        _view_matrix: [[f32; 4]; 4],
        _proj_matrix: [[f32; 4]; 4],
    ) {
        // デフォルト: カメラ更新なし
    }

    /// 状態更新（デフォルトは空）
    fn update(&mut self) {}

    /// Anyトレイトへのダウンキャスト用
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
