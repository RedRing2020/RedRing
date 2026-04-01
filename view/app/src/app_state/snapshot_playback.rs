//! AppState のSnapshot再生・スクラブ制御を扱うモジュール。

use super::debug_snapshot_state::SnapshotPlaybackMode;
use super::AppState;
use crate::selection_rect::SelectionRect;
use std::time::Instant;

const AUTO_PLAY_BASE_DURATION_SEC: f64 = 12.0;
const PLAYBACK_SPEED_MIN: f64 = 0.1;
const PLAYBACK_SPEED_MAX: f64 = 10.0;

impl AppState {
    /// デバッグ用: cam_sim 実行結果をスナップショット系列として読み込む
    pub fn load_debug_simulation_snapshots(&mut self) {
        match viewmodel::snapshot_converter::load_demo_cam_snapshot_domain_series() {
            Ok(series) => {
                let frame_count = series.frames.len();
                self.debug_snapshot.series = Some(series);
                self.debug_snapshot.wireframes = None;
                self.debug_snapshot.solids = None;
                self.debug_snapshot.toolpath_lines = None;
                self.debug_snapshot.tool_lines = None;
                self.debug_snapshot.shaded_mode = false;
                self.debug_snapshot.cursor = 0;
                self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
                self.debug_snapshot.playback_progress = 0.0;
                self.debug_snapshot.last_playback_tick = None;
                tracing::info!(
                    "シミュレーションスナップショット読込完了: {} フレーム",
                    frame_count
                );
                self.log_current_snapshot_frame(true);
            }
            Err(error) => {
                tracing::error!(
                    error_kind = logging_foundation::ERROR_KIND_SIMULATION,
                    "cam simulation snapshot load failed: {}",
                    error
                );
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

        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
        self.debug_snapshot.last_playback_tick = None;
        self.debug_snapshot.cursor = (self.debug_snapshot.cursor + 1) % series.frames.len();
        self.sync_playback_progress_from_cursor();
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

        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
        self.debug_snapshot.last_playback_tick = None;
        self.debug_snapshot.cursor = if self.debug_snapshot.cursor == 0 {
            series.frames.len() - 1
        } else {
            self.debug_snapshot.cursor - 1
        };
        self.sync_playback_progress_from_cursor();
        self.log_current_snapshot_frame(true);
    }

    pub fn toggle_auto_snapshot_playback(&mut self) {
        if self.debug_snapshot.series.is_none() {
            self.load_debug_simulation_snapshots();
        }
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        match self.debug_snapshot.playback_mode {
            SnapshotPlaybackMode::AutoPlaying => {
                self.pause_auto_snapshot_playback();
            }
            SnapshotPlaybackMode::Manual | SnapshotPlaybackMode::Paused => {
                if self.debug_snapshot.playback_progress >= 1.0 {
                    self.debug_snapshot.playback_progress = 0.0;
                    self.set_snapshot_cursor_from_progress(0.0, false);
                }
                self.debug_snapshot.playback_mode = SnapshotPlaybackMode::AutoPlaying;
                self.debug_snapshot.last_playback_tick = Some(Instant::now());
                self.refresh_snapshot_window_title();
                tracing::info!(
                    "Snapshot自動再生を開始: {:.1}x",
                    self.debug_snapshot.playback_speed_factor
                );
            }
        }
    }

    pub(crate) fn pause_auto_snapshot_playback(&mut self) {
        if self.debug_snapshot.series.is_none() {
            self.load_debug_simulation_snapshots();
        }
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }
        if self.debug_snapshot.playback_mode != SnapshotPlaybackMode::AutoPlaying {
            return;
        }

        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Paused;
        self.debug_snapshot.last_playback_tick = None;
        self.refresh_snapshot_window_title();
        tracing::info!(
            "Snapshot自動再生を一時停止: {:.1}x",
            self.debug_snapshot.playback_speed_factor
        );
    }

    pub fn stop_auto_snapshot_playback(&mut self) {
        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
        self.debug_snapshot.last_playback_tick = None;
        self.debug_snapshot.playback_progress = 0.0;
        self.set_snapshot_cursor_from_progress(0.0, false);
        self.log_current_snapshot_frame(true);
        tracing::info!("Snapshot自動再生を停止（先頭へ移動）");
    }

    pub fn adjust_snapshot_playback_speed(&mut self, delta: f64) {
        let next = (self.debug_snapshot.playback_speed_factor + delta)
            .clamp(PLAYBACK_SPEED_MIN, PLAYBACK_SPEED_MAX);
        self.debug_snapshot.playback_speed_factor = next;
        self.refresh_snapshot_window_title();
        tracing::info!(
            "Snapshot再生速度: {:.1}x",
            self.debug_snapshot.playback_speed_factor
        );
    }

    pub fn tick_snapshot_playback(&mut self, now: Instant) {
        if self.debug_snapshot.playback_mode != SnapshotPlaybackMode::AutoPlaying {
            return;
        }
        let Some(_series) = &self.debug_snapshot.series else {
            return;
        };

        let last = self.debug_snapshot.last_playback_tick.unwrap_or(now);
        let dt = (now - last).as_secs_f64();
        self.debug_snapshot.last_playback_tick = Some(now);
        if dt <= 0.0 {
            return;
        }

        let step = (dt * self.debug_snapshot.playback_speed_factor) / AUTO_PLAY_BASE_DURATION_SEC;
        self.debug_snapshot.playback_progress =
            (self.debug_snapshot.playback_progress + step).clamp(0.0, 1.0);
        let reached_end = self.debug_snapshot.playback_progress >= 1.0;

        let prev_cursor = self.debug_snapshot.cursor;
        self.set_snapshot_cursor_from_progress(self.debug_snapshot.playback_progress, false);

        if self.debug_snapshot.cursor != prev_cursor {
            self.debug_snapshot.frame_update_required = true;
        }

        if reached_end {
            self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Paused;
            self.debug_snapshot.last_playback_tick = None;
            self.refresh_snapshot_window_title();
            tracing::info!("Snapshot自動再生が末尾に到達して停止");
        }
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
            let toolpath_lines = self
                .debug_snapshot
                .toolpath_lines
                .clone()
                .unwrap_or_default();
            let tool_lines = if let Some(tool_lines_per_frame) = &self.debug_snapshot.tool_lines {
                let overlay_index = frame_index.min(tool_lines_per_frame.len().saturating_sub(1));
                tool_lines_per_frame[overlay_index].clone()
            } else {
                Vec::new()
            };

            let applied = stage.apply_snapshot_solid_frame(
                &self.graphic.device,
                vertices,
                indices,
                self.snapshot_shaded_color_settings.work_solid_color,
                toolpath_lines,
                tool_lines,
            );

            if !applied {
                return;
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
        let applied = stage.apply_snapshot_wireframe_frame(
            &self.graphic.device,
            snapshot_wireframes.clone(),
            frame_index,
        );
        if !applied {
            tracing::debug!("snapshot wireframe frameの適用対象ステージではありません");
        }
    }

    pub(super) fn log_current_snapshot_frame(&mut self, emit_log: bool) {
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let index = self.debug_snapshot.cursor.min(series.frames.len() - 1);
        let payload = series.frames[index].payload;

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

        self.refresh_snapshot_window_title();

        if emit_log {
            self.sync_snapshot_visual_frame();
        }
    }

    fn refresh_snapshot_window_title(&self) {
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        if series.frames.is_empty() {
            return;
        }

        let frame_count = series.frames.len();
        let index = self.debug_snapshot.cursor.min(series.frames.len() - 1);
        let payload = series.frames[index].payload;
        let mode_label = self.snapshot_playback_mode_label().to_string();
        let speed_factor = self.debug_snapshot.playback_speed_factor;

        self.window.set_title(&format!(
            "RedRing | Snapshot {}/{} | seg={} t={:.3} dist={:.3}mm remain={:.3}mm3 | {} {:.1}x",
            index + 1,
            frame_count,
            payload.segment_index,
            payload.segment_t,
            payload.accumulated_distance_mm,
            payload.remaining_volume_mm3,
            mode_label,
            speed_factor,
        ));
    }

    pub(super) fn snapshot_progress_ratio(&self) -> Option<f32> {
        self.debug_snapshot.series.as_ref()?;
        if self.debug_snapshot.playback_mode == SnapshotPlaybackMode::Manual {
            return self
                .snapshot_distance_progress_from_cursor()
                .map(|p| p as f32);
        }
        Some(self.debug_snapshot.playback_progress as f32)
    }

    fn snapshot_track_rect(&self) -> SelectionRect {
        SelectionRect {
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
        if self.debug_snapshot.playback_mode == SnapshotPlaybackMode::AutoPlaying {
            self.debug_snapshot.last_playback_tick = None;
        }
        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
        let rect = self.snapshot_track_rect();
        let progress = ((x - rect.x) / rect.width).clamp(0.0, 1.0) as f64;
        let next_index = self.index_from_distance_progress(progress).unwrap_or(0);

        if next_index != self.debug_snapshot.cursor {
            self.debug_snapshot.cursor = next_index;
            self.log_current_snapshot_frame(emit_log);
            if !emit_log {
                // ドラッグ中（emit_log=false）もリアルタイムでビジュアルを更新する
                self.sync_snapshot_visual_frame();
            }
        }

        self.sync_playback_progress_from_cursor();
    }

    fn snapshot_playback_mode_label(&self) -> &'static str {
        match self.debug_snapshot.playback_mode {
            SnapshotPlaybackMode::Manual => "MANUAL",
            SnapshotPlaybackMode::AutoPlaying => "PLAY",
            SnapshotPlaybackMode::Paused => "PAUSE",
        }
    }

    fn snapshot_distance_bounds(&self) -> Option<(f64, f64)> {
        let series = self.debug_snapshot.series.as_ref()?;
        let first = series.frames.first()?.payload.accumulated_distance_mm;
        let last = series.frames.last()?.payload.accumulated_distance_mm;
        Some((first, last))
    }

    fn snapshot_distance_progress_from_cursor(&self) -> Option<f64> {
        let series = self.debug_snapshot.series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }
        let index = self
            .debug_snapshot
            .cursor
            .min(series.frames.len().saturating_sub(1));
        let distance = series.frames.get(index)?.payload.accumulated_distance_mm;
        let (min_distance, max_distance) = self.snapshot_distance_bounds()?;
        let span = max_distance - min_distance;
        if span.abs() < f64::EPSILON {
            return Some(1.0);
        }
        Some(((distance - min_distance) / span).clamp(0.0, 1.0))
    }

    fn index_from_distance_progress(&self, progress: f64) -> Option<usize> {
        let series = self.debug_snapshot.series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }
        if series.frames.len() == 1 {
            return Some(0);
        }

        let (min_distance, max_distance) = self.snapshot_distance_bounds()?;
        let span = max_distance - min_distance;
        if span.abs() < f64::EPSILON {
            return Some(
                (progress.clamp(0.0, 1.0) * (series.frames.len() as f64 - 1.0)).round() as usize,
            );
        }

        let target_distance = min_distance + span * progress.clamp(0.0, 1.0);
        let mut best_index = 0usize;
        let mut best_diff = f64::INFINITY;

        for (index, frame) in series.frames.iter().enumerate() {
            let diff = (frame.payload.accumulated_distance_mm - target_distance).abs();
            if diff < best_diff {
                best_diff = diff;
                best_index = index;
            }
        }
        Some(best_index)
    }

    fn set_snapshot_cursor_from_progress(&mut self, progress: f64, emit_log: bool) {
        self.debug_snapshot.playback_progress = progress.clamp(0.0, 1.0);
        let Some(next_index) =
            self.index_from_distance_progress(self.debug_snapshot.playback_progress)
        else {
            return;
        };
        if next_index != self.debug_snapshot.cursor {
            self.debug_snapshot.cursor = next_index;
            self.debug_snapshot.frame_update_required = true;
            self.log_current_snapshot_frame(emit_log);
            if !emit_log {
                self.sync_snapshot_visual_frame();
            }
        }
    }

    fn sync_playback_progress_from_cursor(&mut self) {
        if let Some(progress) = self.snapshot_distance_progress_from_cursor() {
            self.debug_snapshot.playback_progress = progress;
        }
    }
}
