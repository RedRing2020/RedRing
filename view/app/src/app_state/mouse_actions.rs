//! AppState のマウス入力ハンドリングを扱うモジュール。

use super::snapshot_playback::SnapshotLoadTrigger;
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
                    // 回転は、3D の表示の帯の中の位置で決める（帯の中心が回転の中心）
                    let viewport_width = self.graphic.config.width as f32;
                    let (band_top, band_height) = self.view_band();
                    // 帯の外で押したときも、帯の中に寄せる（最初の動きで回転が跳ねないように）
                    self.arcball_virtual_cursor = Some(
                        self.cursor_position
                            .map(|(x, y)| {
                                (
                                    x.clamp(0.0, viewport_width),
                                    (y - band_top).clamp(0.0, band_height),
                                )
                            })
                            .unwrap_or((viewport_width * 0.5, band_height * 0.5)),
                    );
                    tracing::debug!(
                        "Ctrl+left drag: started camera rotation start={:?} current={:?} virtual_start={:?}",
                        self.arcball_drag_start,
                        self.cursor_position,
                        self.arcball_virtual_cursor
                    );
                    return;
                }

                if let Some(cursor) = self.cursor_position {
                    if self.is_cursor_on_snapshot_track(cursor) {
                        if !self.ensure_snapshot_series_ready(SnapshotLoadTrigger::SnapshotScrub) {
                            return;
                        }
                        self.snapshot_scrub_active = true;
                        self.mouse_input.cancel_operation();
                        self.set_snapshot_cursor_from_x(cursor.0, true);
                        tracing::debug!("Left click: started snapshot scrubbing");
                        return;
                    }
                }

                tracing::debug!(
                    "Left drag: view rectangle selection (camera control is Ctrl+drag)"
                );

                if let Some(cursor) = self.cursor_position {
                    self.selection_rect_drag_origin = Some(cursor);
                    self.active_selection_rect = Some(SelectionRect::from_points(cursor, cursor));
                }
            }
            winit::event::ElementState::Released => {
                if self.snapshot_scrub_active {
                    self.snapshot_scrub_active = false;
                    tracing::debug!("Left drag: finished snapshot scrubbing");
                    self.log_current_snapshot_frame(true);
                    return;
                }

                if let Some(start) = self.arcball_drag_start.take() {
                    tracing::debug!(
                        "Ctrl+left drag: finished camera rotation start={:?} end={:?} virtual_end={:?}",
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

    /// 3D の表示で、パンのドラッグをしているか
    pub fn is_view_panning(&self) -> bool {
        self.mouse_input.operation() == crate::mouse_input::MouseOperation::Pan
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

        // パンは、画面の上のカーソルの移動量で動かす（OS のポインターの速さ・加速を含めて、形がカーソルに付いてくる。
        // マウスの生の移動量は、それを含まない）
        if self.is_view_panning() {
            if let Some((last_x, last_y)) = self.last_cursor_position {
                let screen_height = (self.graphic.config.height as f32).max(1.0);
                let aspect = self.graphic.config.width as f32 / screen_height;
                self.camera
                    .pan(x - last_x, y - last_y, aspect, screen_height);
                self.update_camera_uniforms();
            }
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
                let (_, viewport_height) = self.view_band();
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
            // パンは、カーソルの移動（`handle_cursor_moved`）で動かす
            MouseOperation::Pan => {}
            MouseOperation::Zoom => {
                self.camera.zoom(delta_x, delta_y);
                self.update_camera_uniforms();
            }
            MouseOperation::None => {}
        }
    }
}
