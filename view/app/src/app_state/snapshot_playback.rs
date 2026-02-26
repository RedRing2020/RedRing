//! AppState のSnapshot再生・スクラブ制御を扱うモジュール。

use super::AppState;
use crate::view_rect::ViewRect;
use stage::{MeshStage, OctreeStage};

impl AppState {
    /// デバッグ用: cam_sim 実行結果をスナップショット系列として読み込む
    pub fn load_debug_simulation_snapshots(&mut self) {
        match viewmodel::snapshot_converter::create_sample_cam_snapshot_domain_series() {
            Ok(series) => {
                let frame_count = series.frames.len();
                self.debug_snapshot.series = Some(series);
                self.debug_snapshot.wireframes = None;
                self.debug_snapshot.solids = None;
                self.debug_snapshot.toolpath_lines = None;
                self.debug_snapshot.tool_lines = None;
                self.debug_snapshot.shaded_mode = false;
                self.debug_snapshot.cursor = 0;
                tracing::info!(
                    "シミュレーションスナップショット読込完了: {} フレーム",
                    frame_count
                );
                self.log_current_snapshot_frame(true);
            }
            Err(error) => {
                tracing::error!("シミュレーションスナップショット読込失敗: {}", error);
            }
        }
    }

    /// デバッグ用: 次フレームへ進めて内容をログ表示
    pub fn cycle_debug_simulation_snapshot(&mut self) {
        let Some(series) = &self.debug_snapshot.series else {
            self.load_debug_simulation_snapshots();
            return;
        };

        if series.frames.is_empty() {
            tracing::warn!("スナップショット系列が空です");
            return;
        }

        self.debug_snapshot.cursor = (self.debug_snapshot.cursor + 1) % series.frames.len();
        self.log_current_snapshot_frame(true);
    }

    /// デバッグ用: 前フレームへ戻して内容をログ表示
    pub fn rewind_debug_simulation_snapshot(&mut self) {
        let Some(series) = &self.debug_snapshot.series else {
            self.load_debug_simulation_snapshots();
            return;
        };

        if series.frames.is_empty() {
            tracing::warn!("スナップショット系列が空です");
            return;
        }

        self.debug_snapshot.cursor = if self.debug_snapshot.cursor == 0 {
            series.frames.len() - 1
        } else {
            self.debug_snapshot.cursor - 1
        };
        self.log_current_snapshot_frame(true);
    }

    pub(super) fn sync_snapshot_visual_frame(&mut self) {
        if self.debug_snapshot.shaded_mode {
            let Some(snapshot_solids) = &self.debug_snapshot.solids else {
                return;
            };
            if snapshot_solids.is_empty() {
                return;
            }

            let frame_index = self
                .debug_snapshot
                .cursor
                .min(snapshot_solids.len().saturating_sub(1));
            let (vertices, indices) = snapshot_solids[frame_index].clone();

            let stage = self.renderer.get_stage_mut();
            let Some(mesh_stage) = stage.as_any_mut().downcast_mut::<MeshStage>() else {
                return;
            };
            mesh_stage.set_mesh_data(&self.graphic.device, vertices, indices);
            mesh_stage.set_mesh_base_color(self.snapshot_shaded_color_settings.work_solid_color);
            mesh_stage.clear_overlay_line_data();

            let toolpath_lines = self
                .debug_snapshot
                .toolpath_lines
                .clone()
                .unwrap_or_default();
            if let Some(tool_lines_per_frame) = &self.debug_snapshot.tool_lines {
                let overlay_index = frame_index.min(tool_lines_per_frame.len().saturating_sub(1));
                let tool_lines = tool_lines_per_frame[overlay_index].clone();
                if !tool_lines.is_empty() {
                    mesh_stage.set_overlay_tool_line_data(&self.graphic.device, tool_lines);
                }
            }
            if !toolpath_lines.is_empty() {
                mesh_stage.set_overlay_toolpath_line_data(&self.graphic.device, toolpath_lines);
            }
            return;
        }

        let Some(snapshot_wireframes) = &self.debug_snapshot.wireframes else {
            return;
        };
        if snapshot_wireframes.is_empty() {
            return;
        }

        let frame_index = self
            .debug_snapshot
            .cursor
            .min(snapshot_wireframes.len().saturating_sub(1));

        let stage = self.renderer.get_stage_mut();
        let Some(octree_stage) = stage.as_any_mut().downcast_mut::<OctreeStage>() else {
            return;
        };

        if octree_stage.max_depth().saturating_add(1) != snapshot_wireframes.len() {
            octree_stage.set_depth_levels(&self.graphic.device, snapshot_wireframes.clone());
        }
        octree_stage.set_depth(&self.graphic.device, frame_index);
    }

    pub(super) fn log_current_snapshot_frame(&mut self, emit_log: bool) {
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let index = self.debug_snapshot.cursor.min(series.frames.len() - 1);
        let frame = &series.frames[index];
        let payload = frame.payload;

        if emit_log {
            tracing::info!(
                "Snapshot frame {}/{}: seg={}, t={:.3}, dist={:.3}mm, remain={:.3}mm3",
                index + 1,
                series.frames.len(),
                payload.segment_index,
                payload.segment_t,
                payload.accumulated_distance_mm,
                payload.remaining_volume_mm3,
            );
        }

        self.window.set_title(&format!(
            "RedRing | Snapshot {}/{} | seg={} t={:.3} dist={:.3}mm remain={:.3}mm3",
            index + 1,
            series.frames.len(),
            payload.segment_index,
            payload.segment_t,
            payload.accumulated_distance_mm,
            payload.remaining_volume_mm3,
        ));

        self.sync_snapshot_visual_frame();
    }

    pub(super) fn snapshot_progress_ratio(&self) -> Option<f32> {
        let series = self.debug_snapshot.series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }

        Some(
            if series.frames.len() <= 1 {
                1.0
            } else {
                (self.debug_snapshot.cursor as f32) / ((series.frames.len() - 1) as f32)
            }
            .clamp(0.0, 1.0),
        )
    }

    fn snapshot_track_rect(&self) -> ViewRect {
        ViewRect {
            x: 16.0,
            y: 16.0,
            width: 220.0,
            height: 14.0,
        }
    }

    pub(super) fn is_cursor_on_snapshot_track(&self, cursor: (f32, f32)) -> bool {
        self.snapshot_track_rect().contains(cursor)
    }

    pub(super) fn set_snapshot_cursor_from_x(&mut self, x: f32, emit_log: bool) {
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let rect = self.snapshot_track_rect();
        let progress = ((x - rect.x) / rect.width).clamp(0.0, 1.0);
        let next_index = if series.frames.len() <= 1 {
            0
        } else {
            (progress * (series.frames.len() as f32 - 1.0)).round() as usize
        };

        if next_index != self.debug_snapshot.cursor {
            self.debug_snapshot.cursor = next_index;
            self.log_current_snapshot_frame(emit_log);
        }
    }
}
