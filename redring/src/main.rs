use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::error::EventLoopError;
use winit::window::{Window, WindowAttributes};

mod app_renderer;
use crate::app_renderer::{AppRenderer, RenderStage};

mod graphic;
use crate::graphic::init_wgpu;
use std::sync::Arc;
use winit::window::WindowId;

struct App {
    renderer: Option<AppRenderer>,
    window: Option<Arc<Window>>,
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .expect("Failed to create window"),
        );

        let (device, _queue, _surface, config) = init_wgpu(&window);
        let mut _renderer = AppRenderer::new(&device, &config);
        _renderer.set_stage(RenderStage::TwoD);

        self.window = Some(window.clone());
        self.renderer = Some(_renderer);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::RedrawRequested = event {
            if let Some(_renderer) = &mut self.renderer {
                // renderer.render(&device, &queue, &surface, &config); ← 必要なら保持して呼び出し
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::builder().build()?;
    let mut app = App {
        renderer: None,
        window: None,
    };
    event_loop.run_app(&mut app)
}