use wgpu::{CommandEncoder, TextureView};
use crate::render_stage::RenderStage;
use render::pipeline; // render クレートの pipeline モジュールを参照

/// ワイヤーフレーム描画ステージ
pub struct WireframeStage {
    // 将来的に vertex_buffer, pipeline, camera_state などを保持可能
}

impl WireframeStage {
    pub fn new() -> Self {
        Self {
            // 初期化処理（必要なら）
        }
    }
}

impl RenderStage for WireframeStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        pipeline::draw_wireframe(encoder, view);
    }
}