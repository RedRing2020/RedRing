use std::sync::Arc;
use winit::keyboard::{Key, NamedKey};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{WindowAttributes, WindowId},
};

use crate::app_state::AppState;

/// ウィンドウのタイトル（切削シミュレーションを取り込む前と、デバッグ表示の間）
pub(crate) const APP_TITLE: &str = "RedRing";

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
        // 描画の準備ができるまで、OS がウィンドウを既定の背景（白）で塗らないよう、見えない状態で作る
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title(APP_TITLE)
                        .with_visible(false),
                )
                .expect("Window creation failed"),
        );
        let mut state = AppState::new(window);
        // 見えないウィンドウには描画の合図が来ないため、描画面を今の大きさに合わせ、最初のフレームをここで描いてから見せる。
        // egui は最初の描画で大きさを測ることがあるため、2 回描く（大きさ 0 のときは描かずに見せる）
        let size = state.window.inner_size();
        if size.width > 0 && size.height > 0 {
            state.resize(size);
            state.render();
            state.render();
        }
        state.window.set_visible(true);
        self.state = Some(state);
        tracing::info!("=== RedRing is ready ===");
        tracing::info!(
            "Cutting simulation: open the machining sequence with Machining > Open machining sequence and submit from the operation list"
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
                // 保存していない変更を確かめてから終える（確かめはイベントの処理が終わった後）
                WindowEvent::CloseRequested => state.request_exit(),
                WindowEvent::Resized(size) => state.resize(size),
                WindowEvent::RedrawRequested => state.render(),
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = event.state == ElementState::Pressed;
                    state.handle_keyboard_input(
                        &event.logical_key,
                        &event.key_without_modifiers(),
                        pressed,
                    );

                    // Esc はアプリを終える（保存していない変更を確かめてから。押し続けの繰り返しは除く）。条件の編集の
                    // ダイアログを開いている間は、ダイアログを閉じるだけにする
                    if pressed
                        && !event.repeat
                        && !state.is_condition_dialog_open()
                        && matches!(event.logical_key, Key::Named(NamedKey::Escape))
                    {
                        state.request_exit();
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
                // 条件の編集のダイアログを開いている間は、ファイルを受けない（編集している条件を黙って捨てないように）
                WindowEvent::DroppedFile(path) if state.is_condition_dialog_open() => {
                    tracing::info!(path = %path.display(), "File drop ignored while a dialog is open");
                }
                WindowEvent::DroppedFile(path) => state.drop_file(path),
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
            if state.exit_confirmed() {
                self.should_exit = true;
                event_loop.exit();
                return;
            }
            state.tick_snapshot_playback(std::time::Instant::now());
            state.window.request_redraw();
        }
    }
}
