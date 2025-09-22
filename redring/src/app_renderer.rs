use wgpu::{Device, SurfaceConfiguration, CommandEncoder, TextureView};
use stage::{RenderStage, DraftStage, OutlineStage, ShadingStage};

pub struct AppRenderer {
    stage: Box<dyn RenderStage>,
}

impl AppRenderer {
    /// 初期化：Draftステージを生成
    pub fn new_draft(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(DraftStage::new(device, config.format));
        Self { stage }
    }

    /// 初期化：Outlineステージを生成
    pub fn new_outline(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(OutlineStage::new(device, config.format));
        Self { stage }
    }
    
    /// 初期化：Shadingステージを生成
    pub fn new_shading(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(ShadingStage::new(device, config.format));
        Self { stage }
    }

    /// ステージ切り替え（将来的なイベント駆動対応）
    pub fn set_stage(&mut self, stage: Box<dyn RenderStage>) {
        self.stage = stage;
    }

    /// 描画処理：現在のステージに委譲
    pub fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        self.stage.render(encoder, view);
    }
    
    pub fn update(&mut self) {
        self.stage.update();
    }

}