use wgpu::{CommandEncoder, TextureView};

/// 描画ステージの共通インターフェース
pub trait RenderStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView);
}