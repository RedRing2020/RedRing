//! AppState のキーボード入力ハンドリングを扱うモジュール。

use super::AppState;
use super::model_file::FileAction;
use winit::keyboard::Key;

impl AppState {
    /// キーボード入力を処理（`key` は文字としてのキー、`key_without_modifiers` は Ctrl・Shift などを除いたキー）
    pub fn handle_keyboard_input(
        &mut self,
        key: &winit::keyboard::Key,
        key_without_modifiers: &Key,
        pressed: bool,
    ) {
        self.mouse_input.update_key(key, pressed);

        if !pressed {
            return;
        }

        // 条件の編集のダイアログを開いている間は、Esc でダイアログをキャンセルする（入力欄の編集中は、入力欄を離れる
        // だけにする）
        if self.is_condition_dialog_open()
            && matches!(
                key,
                winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape)
            )
        {
            if !self.renderer.is_settings_using_keyboard() {
                self.cancel_condition_dialog();
            }
            return;
        }

        // Ctrl とのファイルの操作の組み合わせは、ほかのキー操作にしない（Ctrl+Shift+S で Shift+S の操作を動かさない）。
        // 入力欄の編集中も受ける（Ctrl との組み合わせは文字を打たない）。AltGr（Ctrl+Alt）は文字を打つため除く。条件の
        // 編集のダイアログを開いている間は、文書を替える・保存する操作をしない（編集している条件と食い違うため）
        if self.mouse_input.is_ctrl_pressed()
            && !self.mouse_input.is_alt_pressed()
            && let Some(action) =
                file_shortcut(key_without_modifiers, self.mouse_input.is_shift_pressed())
        {
            if !self.is_condition_dialog_open() {
                self.request_file_action(action);
            }
            return;
        }

        // 設定パネル・設定のウィンドウの入力欄の編集中は、打った文字をキー操作にしない
        // F1 は文字を打たないため、入力欄の編集中も受ける
        if matches!(
            key,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::F1)
        ) {
            self.toggle_help_controls();
            return;
        }

        if self.renderer.is_settings_using_keyboard() {
            return;
        }

        if matches!(
            key,
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space)
        ) {
            self.toggle_auto_snapshot_playback();
            return;
        }

        if let winit::keyboard::Key::Character(ch) = key {
            match ch.as_str() {
                "M" => {
                    self.toggle_operation_list();
                }
                "L" => {
                    self.toggle_cutting_simulation();
                }
                "r" => {
                    self.camera.reset();
                    self.update_camera_uniforms();
                    tracing::info!("Reset the camera (r key)");
                }
                "t" => {
                    self.camera.reset_to_standard_cad_view();
                    self.update_camera_uniforms();
                    tracing::info!("Set the standard CAD view (t key)");
                }
                "f" => {
                    self.camera.reset_to_front_view();
                    self.update_camera_uniforms();
                    tracing::info!("Set the front view (f key)");
                }
                "e" => {
                    self.camera.emergency_camera_escape();
                    self.update_camera_uniforms();
                    tracing::warn!("Emergency camera escape (e key)");
                }
                "h" => {
                    self.toggle_help_controls();
                }
                "w" => {
                    self.toggle_wireframe();
                }
                "v" => {
                    self.toggle_toolpath_lines();
                }
                "V" => {
                    self.toggle_tool_lines();
                }
                "S" => {
                    self.toggle_settings_panel();
                }
                "k" => {
                    self.cycle_debug_simulation_snapshot();
                }
                "j" => {
                    self.rewind_debug_simulation_snapshot();
                }
                "." => {
                    self.step_tool_to_segment_endpoint(true);
                }
                "," => {
                    self.step_tool_to_segment_endpoint(false);
                }
                "J" => {
                    self.stop_auto_snapshot_playback();
                }
                "+" | "=" => {
                    self.scale_snapshot_playback_speed(2.0);
                }
                "-" | "_" => {
                    self.scale_snapshot_playback_speed(0.5);
                }
                _ => {}
            }
        }
    }

    /// アプリを終える操作（ウィンドウを閉じる・Esc）を、保存していない変更を確かめてから行う
    pub fn request_exit(&mut self) {
        self.request_file_action(FileAction::Exit);
    }
}

/// Ctrl と、修飾を除いたキー `key`（Shift を押しているかが `shift`）の、ファイルの操作（キーの配列に合わせ、打つ文字で
/// 見分ける）
fn file_shortcut(key: &Key, shift: bool) -> Option<FileAction> {
    let Key::Character(text) = key else {
        return None;
    };
    match (text.to_ascii_lowercase().as_str(), shift) {
        ("n", false) => Some(FileAction::New),
        ("o", false) => Some(FileAction::Open),
        ("s", false) => Some(FileAction::Save),
        ("s", true) => Some(FileAction::SaveAs),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_shortcuts_are_told_apart_by_shift() {
        let key = |text: &str| Key::Character(text.into());
        assert_eq!(file_shortcut(&key("s"), false), Some(FileAction::Save));
        assert_eq!(file_shortcut(&key("S"), true), Some(FileAction::SaveAs));
        assert_eq!(file_shortcut(&key("n"), false), Some(FileAction::New));
        assert_eq!(file_shortcut(&key("o"), false), Some(FileAction::Open));
        assert_eq!(file_shortcut(&key("n"), true), None);
        assert_eq!(file_shortcut(&key("m"), false), None);
    }
}
