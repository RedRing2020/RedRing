use winit::event::{DeviceEvent, ElementState, MouseButton};

/// マウス操作の種類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseOperation {
    None,
    Rotate,
    Pan,
    Zoom,
}

/// マウス入力の状態管理
#[derive(Debug, Clone)]
pub struct MouseInput {
    /// 現在の操作種類
    pub operation: MouseOperation,
    /// 前回のマウス位置
    pub last_position: Option<(f32, f32)>,
    /// Ctrlキーが押されているか
    pub ctrl_pressed: bool,
    /// 各マウスボタンの状態
    pub left_pressed: bool,
    pub middle_pressed: bool,
    pub right_pressed: bool,
}

impl MouseInput {
    /// 新しいマウス入力状態を作成
    pub fn new() -> Self {
        Self {
            operation: MouseOperation::None,
            last_position: None,
            ctrl_pressed: false,
            left_pressed: false,
            middle_pressed: false,
            right_pressed: false,
        }
    }

    /// キー状態を更新
    pub fn update_key(&mut self, key: &winit::keyboard::Key, pressed: bool) {
        if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control) = key {
            self.ctrl_pressed = pressed;
            if !pressed {
                self.operation = MouseOperation::None;
                self.last_position = None;
            }
        }
    }

    /// マウスボタン状態を更新
    pub fn update_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let pressed = state == ElementState::Pressed;

        match button {
            MouseButton::Left => {
                self.left_pressed = pressed;
            }
            MouseButton::Middle => {
                self.middle_pressed = pressed;
            }
            MouseButton::Right => {
                self.right_pressed = pressed;
            }
            _ => {}
        }

        // 操作種類を決定
        self.update_operation();

        // ボタンが離された時は位置をリセット
        if !pressed {
            self.last_position = None;
        }
    }

    /// マウス位置を更新してデルタを計算
    pub fn update_mouse_position(&mut self, x: f32, y: f32) -> Option<(f32, f32)> {
        if self.operation == MouseOperation::None {
            return None;
        }

        if let Some((last_x, last_y)) = self.last_position {
            let delta_x = x - last_x;
            let delta_y = y - last_y;
            self.last_position = Some((x, y));
            Some((delta_x, delta_y))
        } else {
            self.last_position = Some((x, y));
            None
        }
    }

    /// デバイスイベントからマウス移動を処理
    pub fn handle_device_event(
        &mut self,
        event: &DeviceEvent,
    ) -> Option<(MouseOperation, f32, f32)> {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if self.operation != MouseOperation::None {
                    Some((self.operation, delta.0 as f32, delta.1 as f32))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 現在の操作種類を更新
    fn update_operation(&mut self) {
        if !self.ctrl_pressed {
            self.operation = MouseOperation::None;
            return;
        }

        self.operation = if self.left_pressed {
            MouseOperation::Rotate
        } else if self.middle_pressed {
            MouseOperation::Pan
        } else if self.right_pressed {
            MouseOperation::Zoom
        } else {
            MouseOperation::None
        };
    }

    /// 操作が有効かどうか
    pub fn is_active(&self) -> bool {
        self.operation != MouseOperation::None
    }
}

impl Default for MouseInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{Key, NamedKey};

    #[test]
    fn test_mouse_input_creation() {
        let input = MouseInput::new();

        assert_eq!(input.operation, MouseOperation::None);
        assert!(!input.ctrl_pressed);
        assert!(!input.left_pressed);
        assert!(!input.middle_pressed);
        assert!(!input.right_pressed);
    }

    #[test]
    fn test_ctrl_key_handling() {
        let mut input = MouseInput::new();

        // Ctrlキーを押す
        input.update_key(&Key::Named(NamedKey::Control), true);
        assert!(input.ctrl_pressed);

        // Ctrlキーを離す
        input.update_key(&Key::Named(NamedKey::Control), false);
        assert!(!input.ctrl_pressed);
        assert_eq!(input.operation, MouseOperation::None);
    }

    #[test]
    fn test_mouse_operations() {
        let mut input = MouseInput::new();

        // Ctrlキーを押してから各マウスボタンをテスト
        input.update_key(&Key::Named(NamedKey::Control), true);

        // 左クリック → 回転
        input.update_mouse_button(MouseButton::Left, ElementState::Pressed);
        assert_eq!(input.operation, MouseOperation::Rotate);

        // 中クリック → パン
        input.update_mouse_button(MouseButton::Left, ElementState::Released);
        input.update_mouse_button(MouseButton::Middle, ElementState::Pressed);
        assert_eq!(input.operation, MouseOperation::Pan);

        // 右クリック → ズーム
        input.update_mouse_button(MouseButton::Middle, ElementState::Released);
        input.update_mouse_button(MouseButton::Right, ElementState::Pressed);
        assert_eq!(input.operation, MouseOperation::Zoom);
    }

    #[test]
    fn test_mouse_position_delta() {
        let mut input = MouseInput::new();

        // 操作を有効にする
        input.update_key(&Key::Named(NamedKey::Control), true);
        input.update_mouse_button(MouseButton::Left, ElementState::Pressed);

        // 初回位置設定
        let delta = input.update_mouse_position(100.0, 100.0);
        assert!(delta.is_none());

        // デルタ計算
        let delta = input.update_mouse_position(110.0, 90.0);
        assert_eq!(delta, Some((10.0, -10.0)));
    }
}
