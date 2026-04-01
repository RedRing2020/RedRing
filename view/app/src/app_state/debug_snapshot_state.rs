use render::vertex_3d::MeshVertex;
use std::time::Instant;
use viewmodel::octree_converter::WireframeVertex;
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
    pub(super) wireframes: Option<Vec<Vec<WireframeVertex>>>,
    pub(super) solids: Option<Vec<(Vec<MeshVertex>, Vec<u32>)>>,
    pub(super) toolpath_lines: Option<Vec<MeshVertex>>,
    pub(super) tool_lines: Option<Vec<Vec<MeshVertex>>>,
    pub(super) shaded_mode: bool,
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
            wireframes: None,
            solids: None,
            toolpath_lines: None,
            tool_lines: None,
            shaded_mode: false,
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
        self.wireframes = None;
        self.solids = None;
        self.toolpath_lines = None;
        self.tool_lines = None;
        self.shaded_mode = false;
        self.cursor = 0;
        self.playback_mode = SnapshotPlaybackMode::Manual;
        self.playback_speed_factor = 1.0;
        self.playback_progress = 0.0;
        self.last_playback_tick = None;
        self.frame_update_required = true;
    }
}
