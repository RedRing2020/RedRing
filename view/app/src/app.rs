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

                    // 従来のキーボード処理も継続
                    if pressed {
                        match &event.logical_key {
                            Key::Character(c) if c.as_str() == "1" => state.set_stage_draft(),
                            Key::Character(c) if c.as_str() == "2" => state.set_stage_outline(),
                            Key::Character(c) if c.as_str() == "3" => state.set_stage_shading(),
                            Key::Character(c) if c.as_str() == "s" => {
                                if let Err(e) = state.load_sample_stl() {
                                    tracing::error!("サンプルSTL読み込み失敗: {}", e);
                                }
                            }
                            Key::Character(c) if c.as_str() == "r" => {
                                state.reset_camera();
                                tracing::info!("カメラリセット");
                            }
                            Key::Character(c) if c.as_str() == "t" => {
                                state.reset_camera_to_safe_view();
                                tracing::info!("安全な視点にリセット");
                            }
                            Key::Character(c) if c.as_str() == "e" => {
                                state.emergency_camera_escape();
                                tracing::warn!("緊急カメラ脱出");
                            }
                            Key::Character(c) if c.as_str() == "h" => {
                                tracing::info!("=== キーバインド ===");
                                tracing::info!("1: ドラフトステージ");
                                tracing::info!("2: アウトラインステージ");
                                tracing::info!("3: シェーディングステージ");
                                tracing::info!("s: サンプルSTL読み込み");
                                tracing::info!("r: カメラリセット（基本位置）");
                                tracing::info!("t: 標準CAD視点にリセット");
                                tracing::info!("e: 緊急脱出（最小距離確保）");
                                tracing::info!("d: カメラ状態ログ出力");
                                tracing::info!("w: ワイヤーフレーム切り替え");
                                tracing::info!("h: このヘルプ");
                                tracing::info!("ESC: アプリ終了");
                            }
                            Key::Character(c) if c.as_str() == "d" => {
                                state.log_camera_state();
                                tracing::info!("カメラ状態ログ出力");
                            }
                            Key::Character(c) if c.as_str() == "w" => {
                                state.toggle_wireframe();
                                tracing::info!("ワイヤーフレーム表示切り替え");
                            }
                            Key::Named(NamedKey::Escape) => {
                                self.should_exit = true;
                                event_loop.exit();
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::MouseInput {
                    button,
                    state: button_state,
                    ..
                } => {
                    state.handle_mouse_button(button, button_state);
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
        if let Some(state) = &mut self.state
            && let DeviceEvent::MouseMotion { delta } = event
        {
            state.handle_mouse_motion(delta);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}
