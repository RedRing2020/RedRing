//! AppState からステージ具体型生成を分離する薄いファクトリ。

use stage::{DraftStage, OutlineStage, RenderStage, ShadingStage};

pub fn create_draft_stage(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Box<dyn RenderStage> {
    Box::new(DraftStage::new(device, format))
}

pub fn create_outline_stage(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Box<dyn RenderStage> {
    Box::new(OutlineStage::new(device, format))
}

pub fn create_shading_stage(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Box<dyn RenderStage> {
    Box::new(ShadingStage::new(device, format))
}
