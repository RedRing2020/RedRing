use wgpu::{Device, SurfaceConfiguration, CommandEncoder, TextureView};
use stage::{RenderStage, Render2DStage, WireframeStage};

pub struct AppRenderer {
    stage: Box<dyn RenderStage>,
}

impl AppRenderer {
    /// 初期化：2Dステージを生成
    pub fn new_2d(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(Render2DStage::new(device, config.format));
        Self { stage }
    }

    /// 初期化：ワイヤーフレームステージを生成
    pub fn new_wireframe(device: &Device, config: &SurfaceConfiguration) -> Self {
        let stage = Box::new(WireframeStage::new(device, config.format));
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