//! cutting_animation - 切削シミュレーションの成果物から、切削アニメーションの描画用のデータを作る
//!
//! 時点を移す場合は、区間が変わった部分のブロックの素材の面だけを、別のスレッドで作り直して渡す（`stock`）。
//! 時点の素材の状態は、成果物の差分のキーフレームと正本から cam_sim の再生の入口
//! （`CuttingSimulationReplay`）で作り直す（材料除去の計算は持たない）。
//! 工具の形とツールパスの線は、ツールパスの成果物と工具から作る。工具は、時点の位置のほか、区間の端点にも置ける
//! （素材は、工具の位置の手前で最も近い時点の状態を表示する）。

mod stock;

use cam_core::{read_toolpath_artifact_v1, BinaryFormatError, Tool, ToolPath};
use cam_sim::{
    collect_toolpath_line_segments, read_cutting_simulation_artifact_v1, SimulationError,
    SimulationSnapshotExport,
};
use geo_algorithms::{
    LineSegment3D, LineSegment3DProperties, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
};

use crate::cam_sim_visualization_converter::{
    create_ball_end_mill_wireframe, create_flat_end_mill_wireframe,
    create_toolpath_wireframe_vertices, tool_tip_position, ToolWireframeVisualizationSettings,
};
use crate::mesh_converter::VertexData;
use crate::octree_converter::WireframeVertex;
use crate::snapshot_converter::{
    cam_snapshot_exports_to_inputs, cam_snapshot_inputs_to_domain_series,
    CamSimulationSnapshotInput, DomainSnapshotSeries,
};
use crate::toolpath_converter::ToolPathVisualizationSettings;
use stock::{StockAnimation, StockWorker};

/// 切削アニメーションを作れない理由
#[derive(Debug)]
pub enum CuttingAnimationError {
    /// 成果物を読めない
    Artifact(BinaryFormatError),
    /// スナップショットがない
    NoSnapshot,
    /// 成果物をツールパスと工具で再生できない
    Replay(SimulationError),
}

impl std::fmt::Display for CuttingAnimationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Artifact(error) => write!(f, "failed to read the artifact: {error}"),
            Self::NoSnapshot => write!(f, "the cutting simulation has no snapshot"),
            Self::Replay(error) => write!(f, "failed to replay the cutting simulation: {error}"),
        }
    }
}

impl std::error::Error for CuttingAnimationError {}

impl From<BinaryFormatError> for CuttingAnimationError {
    fn from(value: BinaryFormatError) -> Self {
        Self::Artifact(value)
    }
}

/// 1 つのブロックの素材の面（描画用の頂点と三角形の頂点番号。三角形がなければ空）
#[derive(Debug, Clone)]
pub struct StockBlockMesh {
    pub block: [usize; 3],
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

/// 切削アニメーションの状態
#[derive(Debug)]
pub struct CuttingAnimation {
    /// 素材の面を別のスレッドで作る係
    stock: StockWorker,
    /// 最初の時点のすべてのブロックの素材の面（取り出すまで持つ）
    initial_stock_blocks: Vec<StockBlockMesh>,
    /// 各時点の位置と統計
    snapshots: Vec<SimulationSnapshotExport>,
    /// 表示している素材の面の時点
    frame: usize,
    /// 区間の端点に置いた工具（端点の番号と、置いたときの時点。時点が変わると解除され、工具は時点の位置に戻る）
    tool_endpoint: Option<(usize, usize)>,
    toolpath: ToolPath<f64>,
    /// 切削シミュレーションと同じ分け方の、ツールパスの直線の区間（スナップショットの区間の番号の対象）
    segments: Vec<(LineSegment3D<f64>, bool)>,
    tool: Tool<f64>,
    /// 描く工具の長さ（工具長を指定しない工具は、最も深い切削の位置で素材の上面に届く長さ。時点によらない）
    drawn_tool_length: f64,
}

impl CuttingAnimation {
    /// ツールパスと切削シミュレーションの成果物と工具と素材の精度から、最初の時点の切削アニメーションを作る
    ///
    /// # Errors
    ///
    /// 成果物を読めない場合・スナップショットがない場合・成果物をツールパスと工具で再生できない場合。
    pub fn new(
        toolpath_artifact: &[u8],
        simulation_artifact: &[u8],
        tool: Tool<f64>,
        precision: f64,
    ) -> Result<Self, CuttingAnimationError> {
        let (_, toolpath) = read_toolpath_artifact_v1(&mut &toolpath_artifact[..])?;
        let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &simulation_artifact[..])?;
        if artifact.snapshots.is_empty() {
            return Err(CuttingAnimationError::NoSnapshot);
        }
        let segments =
            collect_toolpath_line_segments(&toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);
        let drawn_tool_length = tool.cutting_length.unwrap_or_else(|| {
            let lowest_tip = segments
                .iter()
                .filter(|(_, is_cutting)| *is_cutting)
                .flat_map(|(segment, _)| [segment.start_point().z(), segment.end_point().z()])
                .fold(f64::INFINITY, f64::min);
            let stock_top = artifact.grids[2].initial.1;
            // ボールエンドミルの線は、球の中心から長さを測る
            let ball_center = if tool.is_ball_end_mill() {
                tool.radius()
            } else {
                0.0
            };
            stock_top - lowest_tip - ball_center
        });
        let snapshots = artifact.snapshots.clone();
        let stock = StockAnimation::new(artifact, &toolpath, &tool, precision)?;
        Ok(Self {
            initial_stock_blocks: stock.stock_blocks(),
            stock: StockWorker::spawn(stock),
            snapshots,
            frame: 0,
            tool_endpoint: None,
            toolpath,
            segments,
            tool,
            drawn_tool_length,
        })
    }

    /// 時点（スナップショット）の数
    pub fn frame_count(&self) -> usize {
        self.snapshots.len()
    }

    /// 表示している素材の面の時点
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// 各時点の位置と統計の系列
    pub fn snapshot_series(
        &self,
        source: impl Into<String>,
    ) -> DomainSnapshotSeries<CamSimulationSnapshotInput> {
        let inputs = cam_snapshot_exports_to_inputs(&self.snapshots);
        cam_snapshot_inputs_to_domain_series(source, &inputs)
    }

    /// 最初の時点の、すべてのブロックの素材の面を取り出す（三角形のないブロックは含めない。2 回目からは空）
    pub fn take_initial_stock_blocks(&mut self) -> Vec<StockBlockMesh> {
        std::mem::take(&mut self.initial_stock_blocks)
    }

    /// 時点 `frame` の素材の面を、別のスレッドに頼む（範囲外は最後の時点にする）
    ///
    /// できた面は `take_stock_update` で受け取る。受け取る前に次の時点を頼むと、最後に頼んだ時点の面になる。
    pub fn request_frame(&self, frame: usize) {
        self.stock.request(frame);
    }

    /// 頼んだ時点の素材の面ができていれば、前に受け取ってから面が変わりうるブロックの面を受け取り、表示する時点をその時点にする
    ///
    /// 三角形がなくなったブロックは、三角形のない面として返す。まだできていなければ `None`。
    pub fn take_stock_update(&mut self) -> Option<Vec<StockBlockMesh>> {
        let (frame, blocks) = self.stock.take_update()?;
        self.frame = frame;
        Some(blocks)
    }

    /// 区間の端点の数（端点 `e` は、`e = 0` では最初の区間の始点、それ以外は区間 `e - 1` の終点）
    pub fn endpoint_count(&self) -> usize {
        if self.segments.is_empty() {
            0
        } else {
            self.segments.len() + 1
        }
    }

    /// 工具を置いている区間の端点（時点の位置に工具がある場合は `None`）
    pub fn tool_endpoint(&self) -> Option<usize> {
        self.tool_endpoint
            .and_then(|(endpoint, frame)| (frame == self.frame).then_some(endpoint))
    }

    /// 工具を、今の工具の位置から次（`forward`）・前の区間の端点へ置き、素材を表示する時点を返す
    ///
    /// 素材を表示する時点は、その端点の手前で最も近い時点。返した時点の素材の面を受け取ると、工具はその端点に描かれる。
    /// 区間がない場合は `None`。最後の端点より先・最初の端点より前へは移らない。
    pub fn step_tool_endpoint(&mut self, forward: bool) -> Option<usize> {
        let count = self.endpoint_count();
        if count == 0 {
            return None;
        }
        let position = self.tool_endpoint().map_or_else(
            || snapshot_position(&self.snapshots[self.frame]),
            |endpoint| endpoint as f64,
        );
        let endpoint = adjacent_endpoint(position, count, forward);
        let frame = snapshot_before_endpoint(&self.snapshots, endpoint);
        self.tool_endpoint = Some((endpoint, frame));
        Some(frame)
    }

    /// 今の工具の形（区間の端点、または今の時点の位置に置く。位置が分からない場合は空）
    pub fn tool_wireframe(
        &self,
        settings: &ToolWireframeVisualizationSettings,
    ) -> Vec<WireframeVertex> {
        let tip = match self.tool_endpoint() {
            Some(0) => tool_tip_position(&self.segments, 0, 0.0),
            Some(endpoint) => tool_tip_position(&self.segments, endpoint - 1, 1.0),
            None => {
                let snapshot = &self.snapshots[self.frame];
                tool_tip_position(&self.segments, snapshot.segment_index, snapshot.segment_t)
            }
        };
        let Some(tip) = tip else {
            return Vec::new();
        };
        let (radius, length) = (self.tool.radius(), self.drawn_tool_length);
        if self.tool.is_ball_end_mill() {
            create_ball_end_mill_wireframe(tip, radius, length, settings)
        } else {
            create_flat_end_mill_wireframe(tip, radius, length, settings)
        }
    }

    /// ツールパスの線
    pub fn toolpath_wireframe(
        &self,
        settings: &ToolPathVisualizationSettings,
    ) -> Vec<WireframeVertex> {
        create_toolpath_wireframe_vertices(&self.toolpath, settings)
    }
}

/// 時点の、経路の上の位置（区間の番号 + 区間の中の位置。端点 `e` の位置は `e`）
fn snapshot_position(snapshot: &SimulationSnapshotExport) -> f64 {
    snapshot.segment_index as f64 + snapshot.segment_t
}

/// 経路の上の位置 `position` の次（`forward`）・前の区間の端点（端点の数は `count`。両端では端にとどまる）
fn adjacent_endpoint(position: f64, count: usize, forward: bool) -> usize {
    let last = count - 1;
    if forward {
        ((position.floor() as usize) + 1).min(last)
    } else {
        (position.ceil() as usize).saturating_sub(1).min(last)
    }
}

/// 区間の端点 `endpoint` の手前で最も近い時点（端点ちょうどの時点を含む。時点は経路の上の位置の昇順）
fn snapshot_before_endpoint(snapshots: &[SimulationSnapshotExport], endpoint: usize) -> usize {
    snapshots
        .partition_point(|snapshot| snapshot_position(snapshot) <= endpoint as f64)
        .saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    use cam_core::{
        write_toolpath_payload_v1, ArtifactHeaderV1, ArtifactKind, ContourLevelPath,
        CoordinateFrame, CuttingDirection, LengthUnit, PathSegment, SegmentType,
    };
    use cam_sim::{write_cutting_simulation_artifact_v1, CuttingSimulator, SnapshotInterval};
    use geo_algorithms::{Aabb3D, Point3D};

    use super::*;

    pub(super) const PRECISION: f64 = 0.01;

    pub(super) fn tool() -> Tool<f64> {
        Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0)
    }

    /// 素材の上面を 3 本の往復の線で削るツールパスと切削シミュレーションの成果物（テストの間で 1 回だけ作る）
    pub(super) fn artifacts() -> &'static (Vec<u8>, Vec<u8>) {
        static ARTIFACTS: OnceLock<(Vec<u8>, Vec<u8>)> = OnceLock::new();
        ARTIFACTS.get_or_init(|| {
            let points = [
                Point3D::new(-2.0, 1.0, 1.6),
                Point3D::new(32.0, 1.0, 1.6),
                Point3D::new(32.0, 2.0, 1.6),
                Point3D::new(-2.0, 2.0, 1.6),
                Point3D::new(-2.0, 3.0, 1.2),
                Point3D::new(32.0, 3.0, 1.2),
            ];
            let toolpath = ToolPath::new(
                "BEM2".to_string(),
                CuttingDirection::Down,
                vec![],
                vec![ContourLevelPath::new(
                    0,
                    1.6,
                    points
                        .windows(2)
                        .map(|pair| {
                            PathSegment::new_line(
                                pair[0],
                                pair[1],
                                SegmentType::Cutting { feed_rate: 1200.0 },
                            )
                        })
                        .collect(),
                )],
                vec![],
            );
            let mut payload = Vec::new();
            write_toolpath_payload_v1(&mut payload, &toolpath).unwrap();
            let mut toolpath_artifact = Vec::new();
            ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload.len() as u64)
                .write_to(&mut toolpath_artifact)
                .unwrap();
            toolpath_artifact.extend_from_slice(&payload);

            let stock = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(30.0, 4.0, 2.0));
            let mut simulator = CuttingSimulator::new(stock, SnapshotInterval::default());
            simulator.simulate(&toolpath, &tool(), PRECISION).unwrap();
            let mut simulation_artifact = Vec::new();
            write_cutting_simulation_artifact_v1(
                &mut simulation_artifact,
                &simulator.artifact().unwrap(),
                LengthUnit::Millimeter,
                CoordinateFrame::WorldRightHandedZUp,
            )
            .unwrap();
            (toolpath_artifact, simulation_artifact)
        })
    }

    fn animation() -> CuttingAnimation {
        let (toolpath, simulation) = artifacts();
        CuttingAnimation::new(toolpath, simulation, tool(), PRECISION).unwrap()
    }

    /// ブロックごとの面を、頂点の位置と三角形で比べられる形にする
    pub(super) fn positions(
        blocks: &HashMap<[usize; 3], StockBlockMesh>,
    ) -> HashMap<[usize; 3], Vec<[f32; 3]>> {
        blocks
            .iter()
            .filter(|(_, mesh)| !mesh.indices.is_empty())
            .map(|(block, mesh)| {
                let triangles = mesh
                    .indices
                    .iter()
                    .map(|&index| mesh.vertices[index as usize].position)
                    .collect();
                (*block, triangles)
            })
            .collect()
    }

    /// 時点 `frame` を頼み、その時点の素材の面を受け取るまで待つ
    fn show(animation: &mut CuttingAnimation, frame: usize) {
        let started = Instant::now();
        animation.request_frame(frame);
        while animation.take_stock_update().is_none() || animation.frame() != frame {
            assert!(started.elapsed() < Duration::from_secs(60), "frame {frame}");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn frames_follow_the_simulation_snapshots() {
        let mut animation = animation();
        assert!(animation.frame_count() > 2);
        assert_eq!(animation.frame(), 0);
        assert_eq!(
            animation.snapshot_series("cam_sim").frames.len(),
            animation.frame_count()
        );
        assert!(!animation.take_initial_stock_blocks().is_empty());
        assert!(animation.take_initial_stock_blocks().is_empty());
        assert!(!animation
            .toolpath_wireframe(&ToolPathVisualizationSettings::default())
            .is_empty());
    }

    #[test]
    fn tool_wireframe_is_placed_at_the_snapshot_position() {
        let mut animation = animation();
        let settings = ToolWireframeVisualizationSettings::default();
        for frame in 0..animation.frame_count() {
            show(&mut animation, frame);
            let snapshot = animation.snapshots[frame];
            let tip = tool_tip_position(
                &animation.segments,
                snapshot.segment_index,
                snapshot.segment_t,
            )
            .unwrap();
            let wireframe = animation.tool_wireframe(&settings);
            // ボールエンドミルの形の最下点は工具の先端
            let lowest = wireframe
                .iter()
                .map(|vertex| vertex.position[2])
                .fold(f32::INFINITY, f32::min);
            assert!((f64::from(lowest) - tip.z()).abs() < 1e-3, "frame {frame}");
        }
    }

    fn export(segment_index: usize, segment_t: f64) -> SimulationSnapshotExport {
        SimulationSnapshotExport {
            segment_index,
            segment_t,
            accumulated_distance_mm: 0.0,
            remaining_volume_mm3: 0.0,
        }
    }

    #[test]
    fn adjacent_endpoints_step_from_endpoints_and_from_inside_segments() {
        // 端点ちょうどからは隣の端点へ、区間の途中からはその区間の端点へ
        assert_eq!(adjacent_endpoint(2.0, 5, true), 3);
        assert_eq!(adjacent_endpoint(2.0, 5, false), 1);
        assert_eq!(adjacent_endpoint(2.3, 5, true), 3);
        assert_eq!(adjacent_endpoint(2.3, 5, false), 2);
        // 両端では端にとどまる
        assert_eq!(adjacent_endpoint(4.0, 5, true), 4);
        assert_eq!(adjacent_endpoint(0.0, 5, false), 0);
    }

    #[test]
    fn the_stock_is_shown_at_the_last_snapshot_up_to_the_endpoint() {
        let snapshots = [
            export(0, 0.0),
            export(0, 0.6),
            export(1, 1.0),
            export(2, 0.5),
            export(3, 1.0),
        ];
        // 端点の間にある時点は手前の時点、端点ちょうどの時点はその時点
        assert_eq!(snapshot_before_endpoint(&snapshots, 0), 0);
        assert_eq!(snapshot_before_endpoint(&snapshots, 1), 1);
        assert_eq!(snapshot_before_endpoint(&snapshots, 2), 2);
        assert_eq!(snapshot_before_endpoint(&snapshots, 3), 3);
        assert_eq!(snapshot_before_endpoint(&snapshots, 4), 4);
    }

    #[test]
    fn tool_placed_at_an_endpoint_is_drawn_there_until_the_frame_changes() {
        let mut animation = animation();
        let settings = ToolWireframeVisualizationSettings::default();
        let lowest = |animation: &CuttingAnimation| {
            animation
                .tool_wireframe(&settings)
                .iter()
                .map(|vertex| vertex.position[0])
                .fold(f32::INFINITY, f32::min)
        };
        let mut endpoint = 0;
        while endpoint + 1 < animation.endpoint_count() {
            let frame = animation.step_tool_endpoint(true).unwrap();
            show(&mut animation, frame);
            endpoint += 1;
            assert_eq!(animation.tool_endpoint(), Some(endpoint));
            // 素材の時点は、端点の手前
            let snapshot = animation.snapshots[frame];
            assert!(snapshot_position(&snapshot) <= endpoint as f64);
            // 工具は端点（区間 endpoint - 1 の終点）に描かれる
            let tip = tool_tip_position(&animation.segments, endpoint - 1, 1.0).unwrap();
            let radius = animation.tool.radius();
            assert!(
                (f64::from(lowest(&animation)) - (tip.x() - radius)).abs() < 1e-3,
                "endpoint {endpoint}"
            );
        }
        // 最後の端点からは進まない
        let frame = animation.step_tool_endpoint(true).unwrap();
        show(&mut animation, frame);
        assert_eq!(animation.tool_endpoint(), Some(endpoint));
        // 戻ると前の端点へ
        let frame = animation.step_tool_endpoint(false).unwrap();
        show(&mut animation, frame);
        assert_eq!(animation.tool_endpoint(), Some(endpoint - 1));
        // 時点が変わると、工具は時点の位置に戻る
        let other = (animation.frame() + 1) % animation.frame_count();
        show(&mut animation, other);
        assert_eq!(animation.tool_endpoint(), None);
    }

    #[test]
    fn tool_without_a_length_is_drawn_with_the_same_length_in_every_frame() {
        let (toolpath, simulation) = artifacts();
        let mut tool = tool();
        tool.cutting_length = None;
        let mut animation = CuttingAnimation::new(toolpath, simulation, tool, PRECISION).unwrap();
        let settings = ToolWireframeVisualizationSettings::default();
        let mut heights = Vec::new();
        for frame in 0..animation.frame_count() {
            show(&mut animation, frame);
            let wireframe = animation.tool_wireframe(&settings);
            let (lowest, highest) = wireframe.iter().fold(
                (f32::INFINITY, f32::NEG_INFINITY),
                |(lowest, highest), vertex| {
                    (
                        lowest.min(vertex.position[2]),
                        highest.max(vertex.position[2]),
                    )
                },
            );
            heights.push(highest - lowest);
        }
        // 先端の高さ（1.6・1.2）が変わっても、工具の長さは変わらない
        assert!(
            heights
                .iter()
                .all(|height| (height - heights[0]).abs() < 1e-4),
            "{heights:?}"
        );
        // 最も深い切削の位置（先端 1.2）から素材の上面（2.0）までは球の半径より短いため、球の上に線の最小の長さ 1 を描く
        assert!((heights[0] - 2.0).abs() < 1e-4, "{}", heights[0]);
    }

    #[test]
    fn broken_artifacts_are_rejected() {
        let (toolpath, simulation) = artifacts();
        let truncated = &simulation[..simulation.len() - 1];
        assert!(matches!(
            CuttingAnimation::new(toolpath, truncated, tool(), PRECISION),
            Err(CuttingAnimationError::Artifact(_))
        ));
        assert!(matches!(
            CuttingAnimation::new(simulation, simulation, tool(), PRECISION),
            Err(CuttingAnimationError::Artifact(_))
        ));
    }
}
