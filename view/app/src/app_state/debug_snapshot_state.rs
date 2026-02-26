use render::vertex_3d::MeshVertex;
use viewmodel::octree_converter::WireframeVertex;
use viewmodel::snapshot_converter::{CamSimulationSnapshotInput, DomainSnapshotSeries};

#[derive(Default)]
pub(super) struct DebugSnapshotState {
    pub(super) series: Option<DomainSnapshotSeries<CamSimulationSnapshotInput>>,
    pub(super) wireframes: Option<Vec<Vec<WireframeVertex>>>,
    pub(super) solids: Option<Vec<(Vec<MeshVertex>, Vec<u32>)>>,
    pub(super) toolpath_lines: Option<Vec<MeshVertex>>,
    pub(super) tool_lines: Option<Vec<Vec<MeshVertex>>>,
    pub(super) shaded_mode: bool,
    pub(super) cursor: usize,
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
    }
}
