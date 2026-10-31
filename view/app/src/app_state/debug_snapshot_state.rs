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
    /// コマ送り（k キー・j キー）を受け付けるか（`shown_frame` は素材の面を表示している時点。素材の面がなければ `None`）
    ///
    /// 前のコマ送りで移ったコマの素材の面ができて、描くまでは受け付けない。キーを押し続けたときの繰り返しの入力が、
    /// 面の作成より速く溜まってコマを飛ばすのを防ぐ。
    pub(super) fn accepts_frame_step(&self, shown_frame: Option<usize>) -> bool {
        !self.frame_update_required && shown_frame.is_none_or(|frame| frame == self.cursor)
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
        // 最初の描画の前は、まだ面を頼んでいない
        assert!(!state.accepts_frame_step(Some(0)));
        state.frame_update_required = false;
        assert!(state.accepts_frame_step(Some(0)));
        // コマ送りで移ると、面を頼むまでは次のコマ送りを受け付けない
        state.cursor = 1;
        state.frame_update_required = true;
        assert!(!state.accepts_frame_step(Some(0)));
        // 頼んだ後も、移った時点の面を受け取るまでは受け付けない
        state.frame_update_required = false;
        assert!(!state.accepts_frame_step(Some(0)));
        assert!(state.accepts_frame_step(Some(1)));
        // 素材の面がなければ、頼んだ後に受け付ける
        assert!(state.accepts_frame_step(None));
    }
}
