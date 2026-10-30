//! AppState のSnapshot再生・スクラブ制御を扱うモジュール。

use super::debug_snapshot_state::SnapshotPlaybackMode;
use super::AppState;
use crate::selection_rect::SelectionRect;
use std::time::Instant;

/// 1.0 倍の自動再生で、工具が経路に沿って進む速さ（mm/s）
const AUTO_PLAY_TOOL_SPEED_MM_PER_SEC: f64 = 1000.0;
/// 1 回の描画で進める再生の時間の上限（描画が遅れても飛ばさず、再生をゆっくりにする）
const AUTO_PLAY_MAX_TICK_SEC: f64 = 1.0 / 30.0;
const PLAYBACK_SPEED_MIN: f64 = 0.25;
const PLAYBACK_SPEED_MAX: f64 = 32.0;
const SEGMENT_WEIGHT_RATIO: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SnapshotLoadTrigger {
    SnapshotScrub,
    Space,
    KeyK,
    KeyJ,
    NextEndpoint,
    PreviousEndpoint,
    Pause,
}

impl SnapshotLoadTrigger {
    pub(super) fn label(self) -> &'static str {
        match self {
            SnapshotLoadTrigger::SnapshotScrub => "snapshot scrub",
            SnapshotLoadTrigger::Space => "Space",
            SnapshotLoadTrigger::KeyK => "k",
            SnapshotLoadTrigger::KeyJ => "j",
            SnapshotLoadTrigger::NextEndpoint => ".",
            SnapshotLoadTrigger::PreviousEndpoint => ",",
            SnapshotLoadTrigger::Pause => "pause",
        }
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

/// コマ送りで移る先のコマ（今のコマ `cursor` から経路に沿って `step_mm` 以上離れた最初のコマ。最低 1 コマ移る）
///
/// `distances` はコマごとの累積距離（昇順）。進む場合に最後のコマからは最初のコマへ、戻る場合に最初のコマからは
/// 最後のコマへ移る。`step_mm` 以上離れたコマがなければ、最後のコマ・最初のコマへ移る。
fn frame_stepped_by_distance(
    distances: &[f64],
    cursor: usize,
    step_mm: f64,
    forward: bool,
) -> usize {
    let last = distances.len().saturating_sub(1);
    let current = distances[cursor.min(last)];
    if forward {
        if cursor >= last {
            return 0;
        }
        let target = current + step_mm;
        distances
            .partition_point(|&distance| distance < target)
            .clamp(cursor + 1, last)
    } else {
        if cursor == 0 {
            return last;
        }
        let target = current - step_mm;
        distances
            .partition_point(|&distance| distance <= target)
            .saturating_sub(1)
            .min(cursor - 1)
    }
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
        tracing::warn!(
            "切削シミュレーションが取り込まれていません。trigger={}, Shift+L で取り込んでください",
            trigger.label()
        );
        false
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

    /// コマ送りの移動量だけ進む・戻る先のコマ
    fn frame_stepped(&self, forward: bool) -> usize {
        let distances: Vec<f64> = self
            .debug_snapshot
            .series
            .iter()
            .flat_map(|series| &series.frames)
            .map(|frame| frame.payload.accumulated_distance_mm)
            .collect();
        frame_stepped_by_distance(
            &distances,
            self.debug_snapshot.cursor,
            self.debug_snapshot.frame_step_distance_mm,
            forward,
        )
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
        self.debug_snapshot.cursor = self.frame_stepped(true);
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
        self.debug_snapshot.cursor = self.frame_stepped(false);
        self.sync_playback_progress_from_cursor();
        self.log_current_snapshot_frame(true);
    }

    /// 工具を次（`forward`）・前の区間の端点へ移し、素材はその端点の手前の時点を表示する
    pub fn step_tool_to_segment_endpoint(&mut self, forward: bool) {
        let trigger = if forward {
            SnapshotLoadTrigger::NextEndpoint
        } else {
            SnapshotLoadTrigger::PreviousEndpoint
        };
        if !self.ensure_snapshot_series_ready(trigger) {
            return;
        }
        let Some(animation) = &mut self.cutting_animation else {
            return;
        };
        let Some(frame) = animation.step_tool_endpoint(forward) else {
            tracing::warn!("ツールパスに区間がないため、工具を区間の端点へ移せません");
            return;
        };

        self.debug_snapshot.playback_mode = SnapshotPlaybackMode::Manual;
        self.debug_snapshot.last_playback_tick = None;
        self.debug_snapshot.cursor = frame;
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
                    "Snapshot自動再生を開始: {}x",
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
            "Snapshot自動再生を一時停止: {}x",
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

    /// 再生速度を `scale` 倍にする（0.25 倍から 32 倍まで）
    pub fn scale_snapshot_playback_speed(&mut self, scale: f64) {
        let next = (self.debug_snapshot.playback_speed_factor * scale)
            .clamp(PLAYBACK_SPEED_MIN, PLAYBACK_SPEED_MAX);
        self.debug_snapshot.playback_speed_factor = next;
        self.refresh_snapshot_window_title();
        tracing::info!(
            "Snapshot再生速度: {}x",
            self.debug_snapshot.playback_speed_factor
        );
    }

    pub fn tick_snapshot_playback(&mut self, now: Instant) {
        if self.debug_snapshot.playback_mode != SnapshotPlaybackMode::AutoPlaying {
            return;
        }
        let Some(series) = &self.debug_snapshot.series else {
            return;
        };
        // 1.0 倍で全体を再生する時間（経路の距離がなければ進捗を一度に進める）
        let total_distance = series
            .frames
            .last()
            .map_or(0.0, |frame| frame.payload.accumulated_distance_mm);
        let duration = total_distance / AUTO_PLAY_TOOL_SPEED_MM_PER_SEC;

        let last = self.debug_snapshot.last_playback_tick.unwrap_or(now);
        let dt = (now - last).as_secs_f64().min(AUTO_PLAY_MAX_TICK_SEC);
        self.debug_snapshot.last_playback_tick = Some(now);
        if dt <= 0.0 {
            return;
        }

        let step = if duration > 0.0 {
            dt * self.debug_snapshot.playback_speed_factor / duration
        } else {
            1.0
        };
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
        self.debug_snapshot.frame_update_required = false;
        self.sync_cutting_animation_frame();
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

        // 工具の位置（区間の端点に置いたか）は時点を移した後に決まるため、タイトルは移した後に出す
        if emit_log {
            self.sync_snapshot_visual_frame();
        }

        self.refresh_snapshot_window_title();
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

        // 素材の面をまだ今のコマへ移していない間（自動再生で描画を待つ間）は、端点に置いた工具を出さない
        let endpoint = self
            .cutting_animation
            .as_ref()
            .filter(|animation| animation.frame() == index)
            .and_then(|animation| {
                animation
                    .tool_endpoint()
                    .map(|endpoint| (endpoint, animation.endpoint_count()))
            });
        let title = match endpoint {
            Some((endpoint, count)) => format!(
                "RedRing | 工具: 区間の端点 {}/{}（{}） | 素材: Snapshot {}/{}（工具の手前） remain={:.3}mm3 | {} {}x",
                endpoint,
                count - 1,
                if endpoint == 0 {
                    "最初の区間の始点".to_string()
                } else {
                    format!("区間 {} の終点", endpoint - 1)
                },
                index + 1,
                frame_count,
                payload.remaining_volume_mm3,
                mode_label,
                speed_factor,
            ),
            None => format!(
                "RedRing | Snapshot {}/{} | seg={} t={:.3} dist={:.3}mm remain={:.3}mm3 | {} {}x",
                index + 1,
                frame_count,
                payload.segment_index,
                payload.segment_t,
                payload.accumulated_distance_mm,
                payload.remaining_volume_mm3,
                mode_label,
                speed_factor,
            ),
        };
        self.window.set_title(&title);
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
                // ドラッグ中（emit_log=false）は、描き直しのときに 1 回だけ素材の面に反映する
                self.debug_snapshot.frame_update_required = true;
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
    use super::frame_stepped_by_distance;

    const DISTANCES: [f64; 6] = [0.0, 0.3, 0.6, 1.0, 1.0, 2.5];

    #[test]
    fn frame_step_moves_to_the_first_frame_at_least_the_distance_away() {
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 0, 0.5, true), 2);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 0, 0.6, true), 2);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 5, 1.5, false), 4);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 5, 2.0, false), 1);
    }

    #[test]
    fn frame_step_moves_at_least_one_frame() {
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 1, 0.0, true), 2);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 3, 0.0, true), 4);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 3, 0.1, true), 5);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 4, 0.0, false), 3);
    }

    #[test]
    fn frame_step_stops_at_the_ends_and_wraps_from_them() {
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 3, 10.0, true), 5);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 5, 0.5, true), 0);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 2, 10.0, false), 0);
        assert_eq!(frame_stepped_by_distance(&DISTANCES, 0, 0.5, false), 5);
    }
}
