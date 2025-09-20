use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use render::renderer_2d::Renderer2D;

pub enum RenderStage {
    TwoD,
    ThreeD,
}

pub struct AppRenderer {
    stage: RenderStage,
    renderer_2d: Option<Renderer2D>,
    // renderer_3d: Option<Renderer3D>, ← 後で追加
}

impl AppRenderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let renderer_2d = Renderer2D::new(device, config);
        Self {
            stage: RenderStage::TwoD,
            renderer_2d: Some(renderer_2d),
            // renderer_3d: None,
        }
    }

    pub fn render(&mut self, device: &Device, queue: &Queue, surface: &Surface, config: &SurfaceConfiguration) {
        match self.stage {
            RenderStage::TwoD => {
                if let Some(r2d) = &self.renderer_2d {
                    r2d.render(device, queue, surface, config);
                }
            }
            _ => {}
        }
    }

    pub fn set_stage(&mut self, stage: RenderStage) {
        self.stage = stage;
    }

}