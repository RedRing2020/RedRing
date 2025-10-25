use std::sync::Arc;
use winit::keyboard::{Key, NamedKey};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::app_state::AppState;

#[derive(Default)]
pub struct App {
    pub state: Option<AppState>,
    pub should_exit: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default())
                .expect("Window creation failed"),
        );
        self.state = Some(AppState::new(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = &mut self.state {
            match event {
                WindowEvent::CloseRequested => {
                    self.should_exit = true;
                    event_loop.exit();
                }
                WindowEvent::Resized(size) => state.resize(size),
                WindowEvent::RedrawRequested => state.render(),
                WindowEvent::KeyboardInput { event, .. } => match &event.logical_key {
                    Key::Character(c) if c.as_str() == "1" => state.set_stage_draft(),
                    Key::Character(c) if c.as_str() == "2" => state.set_stage_outline(),
                    Key::Character(c) if c.as_str() == "3" => state.set_stage_shading(),
                    Key::Character(c) if c.as_str() == "s" => {
                        if let Err(e) = state.load_sample_stl() {
                            tracing::error!("サンプルSTL読み込み失敗: {}", e);
                        }
                    }
                    Key::Named(NamedKey::Escape) => {
                        self.should_exit = true;
                        event_loop.exit();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}
