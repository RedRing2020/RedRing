//! AppState からステージ具体型生成を分離する薄いファクトリ。

use stage::{DraftStage, RenderStage};

pub fn create_draft_stage(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Box<dyn RenderStage> {
    Box::new(DraftStage::new(device, format))
}
