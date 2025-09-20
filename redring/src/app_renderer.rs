use render::renderer_2d::Renderer2D;

pub enum RenderStage {
    TwoD,
    // 今後追加予定: UI, ThreeD, Selection
}

pub struct AppRenderer {
    renderer_2d: Renderer2D,
    active_stage: RenderStage,
}

impl AppRenderer {
    pub fn new(device: &wgpu::Device, _config: &wgpu::SurfaceConfiguration) -> Self {
        let renderer_2d = Renderer2D::new(device, _config);
        Self {
            renderer_2d,
            active_stage: RenderStage::TwoD,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface: &wgpu::Surface,
        _config: &wgpu::SurfaceConfiguration,
    ) {
        match self.active_stage {
            RenderStage::TwoD => {
                self.renderer_2d.render(device, queue, surface);
            }
        }
    }

    pub fn set_stage(&mut self, stage: RenderStage) {
        self.active_stage = stage;
    }
}