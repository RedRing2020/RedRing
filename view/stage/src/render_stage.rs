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

    /// Device依存の状態更新（デフォルトは空）
    fn update_with_device(&mut self, _device: &wgpu::Device) {
        self.update();
    }

    /// Octree深さを1段進める（未対応ならNone）
    fn cycle_octree_depth(&mut self, _device: &wgpu::Device) -> Option<(usize, usize)> {
        None
    }

    /// Octree深さアニメーション開始（未対応ならfalse）
    fn start_octree_depth_animation(&mut self) -> bool {
        false
    }

    /// ワイヤーフレーム表示切替（未対応ならNone）
    fn toggle_wireframe_mode(&mut self) -> Option<bool> {
        None
    }

    /// メッシュ基本色設定（未対応ならfalse）
    fn set_mesh_base_color(&mut self, _color: [f32; 4]) -> bool {
        false
    }

    /// Snapshotのソリッドフレームを適用（未対応ならfalse）
    fn apply_snapshot_solid_frame(
        &mut self,
        _device: &wgpu::Device,
        _vertices: Vec<render::vertex_3d::MeshVertex>,
        _indices: Vec<u32>,
        _base_color: [f32; 4],
        _toolpath_lines: Vec<render::vertex_3d::MeshVertex>,
        _tool_lines: Vec<render::vertex_3d::MeshVertex>,
    ) -> bool {
        false
    }

    /// Snapshotのワイヤーフレームフレームを適用（未対応ならfalse）
    fn apply_snapshot_wireframe_frame(
        &mut self,
        _device: &wgpu::Device,
        _snapshot_wireframes: Vec<Vec<viewmodel::octree_converter::WireframeVertex>>,
        _frame_index: usize,
    ) -> bool {
        false
    }

    /// Anyトレイトへのダウンキャスト用
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
