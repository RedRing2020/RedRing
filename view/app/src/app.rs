use std::sync::Arc;
use winit::keyboard::{Key, NamedKey};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, WindowEvent},
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
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    state.handle_keyboard_input(&event.logical_key, pressed);

                    // ESCキーのみここで処理（アプリ終了）
                    if pressed && matches!(event.logical_key, Key::Named(NamedKey::Escape)) {
                        self.should_exit = true;
                        event_loop.exit();
                    }
                }
                WindowEvent::MouseInput {
                    button,
                    state: button_state,
                    ..
                } => {
                    state.handle_mouse_button(button, button_state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    state.handle_cursor_moved(position.x as f32, position.y as f32);
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(state) = &mut self.state {
            if let DeviceEvent::MouseMotion { delta } = event {
                state.handle_mouse_motion(delta);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}
