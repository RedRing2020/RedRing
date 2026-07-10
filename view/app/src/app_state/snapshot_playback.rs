//! AppState のSnapshot再生・スクラブ制御を扱うモジュール。

use super::debug_snapshot_state::SnapshotPlaybackMode;
use super::AppState;
use crate::selection_rect::SelectionRect;
use cam_demo::CamSimulationDemoScenario;
use std::time::Instant;

const AUTO_PLAY_BASE_DURATION_SEC: f64 = 12.0;
const PLAYBACK_SPEED_MIN: f64 = 0.1;
const PLAYBACK_SPEED_MAX: f64 = 10.0;
const SEGMENT_WEIGHT_RATIO: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapshotLoadReadiness {
    Ready(CamSimulationDemoScenario),
    NotReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SnapshotLoadTrigger {
    SnapshotScrub,
    Space,
    KeyK,
    KeyJ,
    Pause,
}

impl SnapshotLoadTrigger {
    fn label(self) -> &'static str {
        match self {
            SnapshotLoadTrigger::SnapshotScrub => "snapshot scrub",
            SnapshotLoadTrigger::Space => "Space",
            SnapshotLoadTrigger::KeyK => "k",
            SnapshotLoadTrigger::KeyJ => "j",
            SnapshotLoadTrigger::Pause => "pause",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SnapshotLoadDecision {
    Ready {
        scenario: CamSimulationDemoScenario,
        trigger_label: &'static str,
    },
    NotReady {
        trigger_label: &'static str,
    },
}

fn resolve_snapshot_load_readiness(
    current_cam_demo_scenario: Option<CamSimulationDemoScenario>,
) -> SnapshotLoadReadiness {
    match current_cam_demo_scenario {
        Some(scenario) => SnapshotLoadReadiness::Ready(scenario),
        None => SnapshotLoadReadiness::NotReady,
    }
}

pub(super) fn resolve_snapshot_load_decision(
    trigger: SnapshotLoadTrigger,
    current_cam_demo_scenario: Option<CamSimulationDemoScenario>,
) -> SnapshotLoadDecision {
    let trigger_label = trigger.label();
    match resolve_snapshot_load_readiness(current_cam_demo_scenario) {
        SnapshotLoadReadiness::Ready(scenario) => SnapshotLoadDecision::Ready {
            scenario,
            trigger_label,
        },
        SnapshotLoadReadiness::NotReady => SnapshotLoadDecision::NotReady { trigger_label },
    }
}

fn select_frame_index_by_distance_target(distances: &[f64], target_distance: f64) -> usize {
    if distances.len() <= 1 {
        return 0;
    }

    // Lower-bound keeps cursor progression monotonic during auto playback.
    let mut index = distances.partition_point(|distance| *distance < target_distance);
    if index >= distances.len() {
        return distances.len() - 1;
    }

    // If duplicated distances exist, pick the last duplicate to avoid visual stalls.
    let selected_distance = distances[index];
    while index + 1 < distances.len()
        && (distances[index + 1] - selected_distance).abs() <= f64::EPSILON
    {
        index += 1;
    }

    index
}

fn build_segment_weighted_progress_axis(
    frames: &[viewmodel::snapshot_converter::DomainSnapshotFrame<
        viewmodel::snapshot_converter::CamSimulationSnapshotInput,
    >],
) -> Vec<f64> {
    if frames.is_empty() {
        return Vec::new();
    }
    if frames.len() == 1 {
        return vec![1.0];
    }

    let mut distance_deltas = Vec::with_capacity(frames.len().saturating_sub(1));
    let mut segment_deltas = Vec::with_capacity(frames.len().saturating_sub(1));
    let mut positive_distance_sum = 0.0;
    let mut positive_count = 0usize;
    for pair in frames.windows(2) {
        let prev = pair[0].payload;
        let curr = pair[1].payload;

        let distance_delta = (curr.accumulated_distance_mm - prev.accumulated_distance_mm).max(0.0);
        if distance_delta > f64::EPSILON {
            positive_distance_sum += distance_delta;
            positive_count += 1;
        }

        let prev_segment_pos = prev.segment_index as f64 + prev.segment_t.clamp(0.0, 1.0);
        let curr_segment_pos = curr.segment_index as f64 + curr.segment_t.clamp(0.0, 1.0);
        let segment_delta = (curr_segment_pos - prev_segment_pos).max(0.0);

        distance_deltas.push(distance_delta);
        segment_deltas.push(segment_delta);
    }

    let mean_positive_distance_delta = if positive_count == 0 {
        0.0
    } else {
        positive_distance_sum / positive_count as f64
    };

    let segment_weight_mm = mean_positive_distance_delta * SEGMENT_WEIGHT_RATIO;
    let mut cumulative = Vec::with_capacity(frames.len());
    cumulative.push(0.0);
    let mut running = 0.0;

    for (distance_delta, segment_delta) in distance_deltas.into_iter().zip(segment_deltas) {
        // Segment progression adds a small contribution to smooth perceived speed near boundaries.
        let weighted_delta = distance_delta + segment_weight_mm * segment_delta;
        running += weighted_delta;
        cumulative.push(running);
    }

    if running.abs() <= f64::EPSILON {
        return (0..frames.len())
            .map(|index| index as f64 / (frames.len() as f64 - 1.0))
            .collect();
    }

    cumulative
        .into_iter()
        .map(|value| value / running)
        .collect()
}

impl AppState {
    pub(super) fn ensure_snapshot_series_ready(&mut self, trigger: SnapshotLoadTrigger) -> bool {
        if self.debug_snapshot.series.is_some() {
            return true;
        }

        let (scenario, trigger_label) = match resolve_snapshot_load_decision(
            trigger,
            self.current_cam_demo_scenario,
        ) {
            SnapshotLoadDecision::Ready {
                scenario,
                trigger_label,
            } => (scenario, trigger_label),
            SnapshotLoadDecision::NotReady { trigger_label } => {
                tracing::warn!(
                    "CAMシミュレーションデモが未開始です。trigger={trigger_label}, Shift+B または Shift+F で開始してください"
                );
                return false;
            }
        };

        self.load_sample_toolpath_with_scenario(scenario, trigger_label);
        self.debug_snapshot.series.is_some()
    }

    pub(super) fn rebuild_snapshot_weighted_progress_axis_cache(&mut self) {
        let Some(series) = &self.debug_snapshot.series else {
            self.debug_snapshot.weighted_progress_axis = None;
            return;
        };

        if series.frames.is_empty() {
            self.debug_snapshot.weighted_progress_axis = None;
            return;
        }

        let axis = build_segment_weighted_progress_axis(&series.frames);
        self.debug_snapshot.weighted_progress_axis = Some(axis);
    }

    /// デバッグ用: 次フレームへ進めて内容をログ表示
    pub fn cycle_debug_simulation_snapshot(&mut self) {
        if !self.ensure_snapshot_series_ready(SnapshotLoadTrigger::KeyK) {
            return;
        }

        let Some(series) = &self.debug_snapshot.series else {
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
        if !self.ensure_snapshot_series_ready(SnapshotLoadTrigger::KeyJ) {
            return;
        }

        let Some(series) = &self.debug_snapshot.series else {
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
        if !self.ensure_snapshot_series_ready(SnapshotLoadTrigger::Space) {
            return;
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
        if !self.ensure_snapshot_series_ready(SnapshotLoadTrigger::Pause) {
            return;
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
            if let Some(series) = &self.debug_snapshot.series {
                if !series.frames.is_empty() {
                    self.debug_snapshot.cursor = series.frames.len() - 1;
                    self.debug_snapshot.frame_update_required = true;
                    self.sync_snapshot_visual_frame();
                }
            }
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
        // Keep overlay in sync with the currently displayed frame.
        self.snapshot_weighted_progress_from_cursor()
            .map(|p| p as f32)
            .or(Some(self.debug_snapshot.playback_progress as f32))
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
        let next_index = self.index_from_progress(progress).unwrap_or(0);

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

    fn snapshot_weighted_progress_axis(&self) -> Option<&[f64]> {
        self.debug_snapshot.weighted_progress_axis.as_deref()
    }

    fn snapshot_weighted_progress_from_cursor(&self) -> Option<f64> {
        let series = self.debug_snapshot.series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }
        let axis = self.snapshot_weighted_progress_axis()?;
        if axis.is_empty() {
            return None;
        }
        let index = self
            .debug_snapshot
            .cursor
            .min(series.frames.len().saturating_sub(1));
        axis.get(index)
            .copied()
            .map(|progress| progress.clamp(0.0, 1.0))
    }

    fn index_from_progress(&self, progress: f64) -> Option<usize> {
        let series = self.debug_snapshot.series.as_ref()?;
        if series.frames.is_empty() {
            return None;
        }
        if series.frames.len() == 1 {
            return Some(0);
        }

        let axis = self.snapshot_weighted_progress_axis()?;
        if axis.is_empty() {
            return None;
        }

        let target_progress = progress.clamp(0.0, 1.0);
        Some(select_frame_index_by_distance_target(axis, target_progress))
    }

    fn set_snapshot_cursor_from_progress(&mut self, progress: f64, emit_log: bool) {
        self.debug_snapshot.playback_progress = progress.clamp(0.0, 1.0);
        let Some(next_index) = self.index_from_progress(self.debug_snapshot.playback_progress)
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
        if let Some(progress) = self.snapshot_weighted_progress_from_cursor() {
            self.debug_snapshot.playback_progress = progress;
        }
    }
}

#[cfg(test)]
mod tests {
    use cam_demo::CamSimulationDemoScenario;

    use super::{
        resolve_snapshot_load_decision, resolve_snapshot_load_readiness, SnapshotLoadDecision,
        SnapshotLoadReadiness, SnapshotLoadTrigger,
    };

    #[test]
    fn resolve_snapshot_load_readiness_returns_not_ready_when_scenario_is_absent() {
        assert_eq!(
            resolve_snapshot_load_readiness(None),
            SnapshotLoadReadiness::NotReady
        );
    }

    #[test]
    fn resolve_snapshot_load_readiness_returns_ready_when_scenario_exists() {
        assert_eq!(
            resolve_snapshot_load_readiness(Some(CamSimulationDemoScenario::SuccessFlatEndMill)),
            SnapshotLoadReadiness::Ready(CamSimulationDemoScenario::SuccessFlatEndMill)
        );
    }

    #[test]
    fn resolve_snapshot_load_decision_returns_not_ready_for_all_ui_triggers_without_scenario() {
        let triggers = [
            SnapshotLoadTrigger::SnapshotScrub,
            SnapshotLoadTrigger::Space,
            SnapshotLoadTrigger::KeyK,
            SnapshotLoadTrigger::KeyJ,
            SnapshotLoadTrigger::Pause,
        ];

        for trigger in triggers {
            assert_eq!(
                resolve_snapshot_load_decision(trigger, None),
                SnapshotLoadDecision::NotReady {
                    trigger_label: trigger.label()
                }
            );
        }
    }

    #[test]
    fn resolve_snapshot_load_decision_returns_ready_for_all_ui_triggers_with_scenario() {
        let triggers = [
            SnapshotLoadTrigger::SnapshotScrub,
            SnapshotLoadTrigger::Space,
            SnapshotLoadTrigger::KeyK,
            SnapshotLoadTrigger::KeyJ,
            SnapshotLoadTrigger::Pause,
        ];

        for trigger in triggers {
            assert_eq!(
                resolve_snapshot_load_decision(trigger, Some(CamSimulationDemoScenario::Success)),
                SnapshotLoadDecision::Ready {
                    scenario: CamSimulationDemoScenario::Success,
                    trigger_label: trigger.label()
                }
            );
        }
    }
}
