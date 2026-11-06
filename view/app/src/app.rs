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

impl App {
    /// 描画の装置が失われていれば、知らせの画面で伝えて終了する（描画もバッファの作成もできないため）
    fn exit_if_device_lost(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let Some(state) = &self.state else {
            return false;
        };
        if !state.graphic.is_device_lost() {
            return false;
        }
        if !self.should_exit {
            self.should_exit = true;
            crate::fatal_error_dialog::show_device_lost_dialog();
            event_loop.exit();
        }
        true
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("RedRing"))
                .expect("Window creation failed"),
        );
        self.state = Some(AppState::new(window));
        tracing::info!("=== RedRing is ready ===");
        tracing::info!(
            "Cutting simulation: open the machining conditions window with Shift+M and submit with Submit"
        );
        tracing::info!("Open the controls list with F1 / h or Help > Controls");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.exit_if_device_lost(event_loop) {
            return;
        }
        if let Some(state) = &mut self.state {
            // メニューバーは常に出ているため、egui には常にイベントを渡す。egui のウィンドウ・メニューの上の
            // ポインタの操作は、3D の表示に渡さない
            state.handle_settings_window_event(&event);
            let settings_ui_consumed = state.ui_wants_pointer();
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
                WindowEvent::ModifiersChanged(modifiers) => {
                    state.mouse_input.update_modifiers(modifiers.state());
                }
                WindowEvent::MouseInput {
                    button,
                    state: button_state,
                    ..
                } if !settings_ui_consumed => {
                    state.handle_mouse_button(button, button_state);
                }
                WindowEvent::CursorMoved { position, .. } if !settings_ui_consumed => {
                    state.handle_cursor_moved(position.x as f32, position.y as f32);
                }
                WindowEvent::MouseWheel { delta, .. } if !settings_ui_consumed => {
                    state.handle_mouse_wheel(delta);
                }
                WindowEvent::DroppedFile(path) => state.import_product_shape(&path),
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
            if state.graphic.is_device_lost() {
                return;
            }
            if let DeviceEvent::MouseMotion { delta } = event {
                state.handle_settings_mouse_motion(delta);
                state.handle_mouse_motion(delta);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.exit_if_device_lost(event_loop) {
            return;
        }
        if let Some(state) = &mut self.state {
            // ファイルを開く画面は描画を止めるため、描画の途中ではなく、イベントの処理が終わった後に開く
            state.run_requested_menu_actions();
            state.tick_snapshot_playback(std::time::Instant::now());
            state.window.request_redraw();
        }
    }
}
