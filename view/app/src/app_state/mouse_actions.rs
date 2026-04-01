//! AppState のマウス入力ハンドリングを扱うモジュール。

use super::AppState;
use crate::selection_rect::SelectionRect;

impl AppState {
    /// マウスボタン入力を処理
    pub fn handle_mouse_button(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        self.mouse_input.update_mouse_button(button, state);

        if button != winit::event::MouseButton::Left {
            return;
        }

        match state {
            winit::event::ElementState::Pressed => {
                if self.mouse_input.is_ctrl_pressed() {
                    self.arcball_drag_start = self.cursor_position;
                    let viewport_width = self.graphic.config.width as f32;
                    let viewport_height = self.graphic.config.height as f32;
                    self.arcball_virtual_cursor = Some(
                        self.cursor_position
                            .unwrap_or((viewport_width * 0.5, viewport_height * 0.5)),
                    );
                    tracing::info!(
                        "Ctrl+左ドラッグ: カメラ回転モード開始 start={:?} current={:?} virtual_start={:?}",
                        self.arcball_drag_start,
                        self.cursor_position,
                        self.arcball_virtual_cursor
                    );
                    return;
                }

                if let Some(cursor) = self.cursor_position {
                    if self.is_cursor_on_snapshot_track(cursor) {
                        if self.debug_snapshot.series.is_none() {
                            self.load_debug_simulation_snapshots();
                        }
                        self.snapshot_scrub_active = true;
                        self.mouse_input.cancel_operation();
                        self.set_snapshot_cursor_from_x(cursor.0, true);
                        tracing::debug!("左クリック: snapshotスクラブ開始");
                        return;
                    }
                }

                tracing::info!("左ドラッグ: ビュー矩形選択モード（カメラ操作はCtrl+ドラッグ）");

                if let Some(cursor) = self.cursor_position {
                    self.selection_rect_drag_origin = Some(cursor);
                    self.active_selection_rect = Some(SelectionRect::from_points(cursor, cursor));
                }
            }
            winit::event::ElementState::Released => {
                if self.snapshot_scrub_active {
                    self.snapshot_scrub_active = false;
                    tracing::debug!("左ドラッグ: snapshotスクラブ終了");
                    self.log_current_snapshot_frame(true);
                    return;
                }

                if let Some(start) = self.arcball_drag_start.take() {
                    tracing::info!(
                        "Ctrl+左ドラッグ: カメラ回転モード終了 start={:?} end={:?} virtual_end={:?}",
                        start,
                        self.cursor_position,
                        self.arcball_virtual_cursor
                    );
                }
                self.arcball_virtual_cursor = None;

                if self.selection_rect_drag_origin.take().is_some() {
                    if let Some(rect) = self.active_selection_rect.take() {
                        if !rect.is_empty() {
                            self.last_selection_rect = Some(rect);
                        }
                    }
                }
            }
        }
    }

    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        self.last_cursor_position = self.cursor_position;
        self.cursor_position = Some((x, y));

        if self.snapshot_scrub_active {
            self.set_snapshot_cursor_from_x(x, false);
            return;
        }

        if self.mouse_input.operation() == crate::mouse_input::MouseOperation::Rotate {
            tracing::debug!(
                "🖱️ CursorMoved(rotate): last={:?} current=({:.1},{:.1}) start={:?}",
                self.last_cursor_position,
                x,
                y,
                self.arcball_drag_start
            );
        }

        if let Some(origin) = self.selection_rect_drag_origin {
            self.active_selection_rect = Some(SelectionRect::from_points(origin, (x, y)));
        }
    }

    /// Ctrl+ホイールでズーム
    pub fn handle_mouse_wheel(&mut self, delta: winit::event::MouseScrollDelta) {
        if !self.mouse_input.is_ctrl_pressed() {
            return;
        }

        let sensitivity = self.viewing_operation_settings.camera_control_sensitivity;
        let scroll_y = match delta {
            winit::event::MouseScrollDelta::LineDelta(_, y) => y,
            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                pos.y as f32 * sensitivity.wheel_pixel_to_line
            }
        };

        if scroll_y.abs() < 1e-6 {
            return;
        }

        self.camera.zoom_wheel(scroll_y);
        self.update_camera_uniforms();

        tracing::debug!(
            "🖱️ MouseWheel(zoom): ctrl=true, scroll_y={:.3}, camera_distance={:.3}, bounds={:?}",
            scroll_y,
            self.camera.distance,
            self.camera.orthographic_bounds
        );
    }

    /// マウス移動を処理
    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        use crate::mouse_input::MouseOperation;

        let (delta_x, delta_y) = (delta.0 as f32, delta.1 as f32);

        match self.mouse_input.operation() {
            MouseOperation::Rotate => {
                let viewport_width = self.graphic.config.width as f32;
                let viewport_height = self.graphic.config.height as f32;
                let (prev_x, prev_y) = self
                    .arcball_virtual_cursor
                    .unwrap_or((viewport_width * 0.5, viewport_height * 0.5));

                let curr_x = (prev_x + delta_x).clamp(0.0, viewport_width);
                let curr_y = (prev_y + delta_y).clamp(0.0, viewport_height);
                self.arcball_virtual_cursor = Some((curr_x, curr_y));

                tracing::debug!(
                    "🎯 MouseMotion(rotate): delta=({:.2},{:.2}) start={:?} last={:?} current={:?} virtual_prev=({:.1},{:.1}) virtual_curr=({:.1},{:.1}) center=({:.1},{:.1})",
                    delta_x,
                    delta_y,
                    self.arcball_drag_start,
                    self.last_cursor_position,
                    self.cursor_position,
                    prev_x,
                    prev_y,
                    curr_x,
                    curr_y,
                    viewport_width * 0.5,
                    viewport_height * 0.5
                );
                self.camera.rotate_arcball(
                    prev_x,
                    prev_y,
                    curr_x,
                    curr_y,
                    viewport_width,
                    viewport_height,
                );
                self.update_camera_uniforms();
            }
            MouseOperation::Pan => {
                self.camera.pan(delta_x, delta_y);
                self.update_camera_uniforms();
            }
            MouseOperation::Zoom => {
                self.camera.zoom(delta_x, delta_y);
                self.update_camera_uniforms();
            }
            MouseOperation::None => {}
        }
    }
}
