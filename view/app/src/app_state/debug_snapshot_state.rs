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
            last_playback_tick: None,
            frame_update_required: true,
        }
    }
}

impl DebugSnapshotState {
    pub(super) fn clear(&mut self) {
        self.series = None;
        self.weighted_progress_axis = None;
        self.cursor = 0;
        self.playback_mode = SnapshotPlaybackMode::Manual;
        self.playback_speed_factor = 1.0;
        self.playback_progress = 0.0;
        self.last_playback_tick = None;
        self.frame_update_required = true;
    }
}
