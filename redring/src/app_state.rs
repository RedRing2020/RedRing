use std::sync::Arc;
use winit::window::Window;
use crate::graphic::{Graphic, init_graphic};
use crate::app_renderer::AppRenderer;
use stage::{Render2DStage, WireframeStage};

pub struct AppState {
    pub window: Arc<Window>,
    pub graphic: Graphic,
    pub renderer: AppRenderer,
}

impl AppState {
    pub fn new(window: Arc<Window>) -> Self {
        let graphic = init_graphic(window.clone());
        let renderer = AppRenderer::new_2d(&graphic.device, &graphic.config);

        Self {
            window,
            graphic,
            renderer,
        }
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.graphic.config.width = size.width;
        self.graphic.config.height = size.height;
        self.graphic.surface.configure(&self.graphic.device, &self.graphic.config);
    }

    pub fn render(&mut self) {
        self.graphic.render(&mut self.renderer);
    }

    pub fn set_stage_2d(&mut self) {
        let stage = Box::new(Render2DStage::new(&self.graphic.device, self.graphic.config.format));
        self.renderer.set_stage(stage);
    }

    pub fn set_stage_wireframe(&mut self) {
        let stage = Box::new(WireframeStage::new(&self.graphic.device, self.graphic.config.format));
        self.renderer.set_stage(stage);
    }
}