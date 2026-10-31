use std::time::Instant;
use viewmodel::snapshot_converter::{CamSimulationSnapshotInput, DomainSnapshotSeries};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum SnapshotPlaybackMode {
    #[default]
    Manual,
    AutoPlaying,
    Paused,
}

pub(super) struct DebugSnapshotState {
    pub(super) series: Option<DomainSnapshotSeries<CamSimulationSnapshotInput>>,
    pub(super) weighted_progress_axis: Option<Vec<f64>>,
    pub(super) cursor: usize,
    pub(super) playback_mode: SnapshotPlaybackMode,
    pub(super) playback_speed_factor: f64,
    pub(super) playback_progress: f64,
    /// コマ送り（k キー・j キー）で経路に沿って移る距離（mm）
    pub(super) frame_step_distance_mm: f64,
    pub(super) last_playback_tick: Option<Instant>,
    pub(super) frame_update_required: bool,
}

impl Default for DebugSnapshotState {
    fn default() -> Self {
        Self {
            series: None,
            weighted_progress_axis: None,
            cursor: 0,
            playback_mode: SnapshotPlaybackMode::Manual,
            playback_speed_factor: 1.0,
            playback_progress: 0.0,
            frame_step_distance_mm: 0.0,
            last_playback_tick: None,
            frame_update_required: true,
        }
    }
}

impl DebugSnapshotState {
    /// コマ送り（k キー・j キー）を受け付けるか
    ///
    /// 前のコマ送りで移ったコマを描くまでは受け付けない。キーを押し続けたときの繰り返しの入力が、描画より速く
    /// 溜まってコマを飛ばすのを防ぐ。
    pub(super) fn accepts_frame_step(&self) -> bool {
        !self.frame_update_required
    }

    pub(super) fn clear(&mut self) {
        self.series = None;
        self.weighted_progress_axis = None;
        self.cursor = 0;
        self.playback_mode = SnapshotPlaybackMode::Manual;
        self.playback_speed_factor = 1.0;
        self.playback_progress = 0.0;
        self.frame_step_distance_mm = 0.0;
        self.last_playback_tick = None;
        self.frame_update_required = true;
    }
}

#[cfg(test)]
mod tests {
    use super::DebugSnapshotState;

    #[test]
    fn frame_steps_wait_until_the_previous_step_is_drawn() {
        let mut state = DebugSnapshotState::default();
        // 最初の描画の前は、まだ面を作っていない
        assert!(!state.accepts_frame_step());
        state.frame_update_required = false;
        assert!(state.accepts_frame_step());
        // コマ送りで移ると、描くまでは次のコマ送りを受け付けない
        state.frame_update_required = true;
        assert!(!state.accepts_frame_step());
    }
}
