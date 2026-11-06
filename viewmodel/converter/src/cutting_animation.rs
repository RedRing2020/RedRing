//! cutting_animation - 切削シミュレーションの再生から、切削アニメーションの描画用のデータを作る
//!
//! 時点を移す場合は、区間が変わった部分のブロックの素材の面だけを、別のスレッドで作り直して渡す（`stock`）。
//! 時点の素材の面とツールパスの直線の区間を、描画用の頂点に詰め替える。
//! 工具は、時点の位置のほか、区間の端点にも置ける（素材は、工具の位置の手前で最も近い時点の状態を表示する）。
//! 最後の時点では、素材の面の三角形を、製品形状との比較（削り込み・トレランスの中・削り残し・比べていない）で分けて
//! 渡せる（`set_deviations_shown`）。

mod stock;

pub use application::cutting_simulation_playback::DeviationClass;
use application::cutting_simulation_playback::{
    CuttingSimulationPlayback, SimulationSnapshotExport,
};
use application::machining_sequence::Tool;
use application::toolpath::ToolPath;
use geo_algorithms::{LineSegment3D, LineSegment3DProperties};

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
use stock::{block_mesh, StockWorker};

/// 1 つのブロックの素材の面（描画用の頂点と三角形の頂点番号・分類。三角形がなければ空）
#[derive(Debug, Clone)]
pub struct StockBlockMesh {
    pub block: [usize; 3],
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    /// 三角形ごとの、製品形状との比較による分類（比べない面は、すべて `NotCompared`）
    pub classes: Vec<DeviationClass>,
}

impl StockBlockMesh {
    /// 分類ごとの面（分類の順に 4 つ。使う頂点だけを持ち、その分類の三角形がなければ空）
    pub fn into_parts(self) -> [(DeviationClass, Vec<VertexData>, Vec<u32>); 4] {
        // 分けない面（再生中のほとんどの時点）は、写さずにそのまま渡す
        if self
            .classes
            .iter()
            .all(|&class| class == DeviationClass::NotCompared)
        {
            return [
                (DeviationClass::NotCompared, self.vertices, self.indices),
                (DeviationClass::Uncut, Vec::new(), Vec::new()),
                (DeviationClass::Within, Vec::new(), Vec::new()),
                (DeviationClass::Gouge, Vec::new(), Vec::new()),
            ];
        }
        [
            DeviationClass::NotCompared,
            DeviationClass::Uncut,
            DeviationClass::Within,
            DeviationClass::Gouge,
        ]
        .map(|class| {
            let mut remap = vec![u32::MAX; self.vertices.len()];
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            for (triangle, _) in self
                .indices
                .chunks(3)
                .zip(&self.classes)
                .filter(|(_, &triangle_class)| triangle_class == class)
            {
                for &index in triangle {
                    let slot = &mut remap[index as usize];
                    if *slot == u32::MAX {
                        *slot = vertices.len() as u32;
                        vertices.push(self.vertices[index as usize]);
                    }
                    indices.push(*slot);
                }
            }
            (class, vertices, indices)
        })
    }
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
    /// 最後の時点の素材の面を、製品形状との比較で分けるか
    deviations_shown: bool,
    /// 最後に頼んだ時点
    requested: std::cell::Cell<usize>,
}

impl CuttingAnimation {
    /// 切削シミュレーションの再生 `playback` と工具 `tool` から、最初の時点の切削アニメーションを作る
    pub fn new(playback: CuttingSimulationPlayback, tool: Tool<f64>) -> Self {
        let CuttingSimulationPlayback {
            toolpath,
            segments,
            snapshots,
            stock_top,
            stock,
        } = playback;
        let drawn_tool_length = drawn_tool_length(&tool, &segments, stock_top);
        Self {
            initial_stock_blocks: stock
                .stock_blocks()
                .into_iter()
                .map(|(block, mesh)| block_mesh(block, mesh))
                .collect(),
            stock: StockWorker::spawn(stock),
            snapshots,
            frame: 0,
            tool_endpoint: None,
            toolpath,
            segments,
            tool,
            drawn_tool_length,
            deviations_shown: false,
            requested: std::cell::Cell::new(0),
        }
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
        self.requested.set(frame);
        self.stock.request(frame, self.deviations_shown);
    }

    /// 最後の時点の素材の面を、製品形状との比較で分けるか（`shown`）を決める（成果物に比較がなければ分けない）
    ///
    /// 変わった場合、最後に頼んだ時点が最後の時点なら、その時点の素材の面を頼み直す（すべてのブロックの面が作り直される）。
    pub fn set_deviations_shown(&mut self, shown: bool) {
        if self.deviations_shown != shown {
            self.deviations_shown = shown;
            let requested = self.requested.get();
            if requested >= self.frame_count() - 1 {
                self.request_frame(requested);
            }
        }
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

/// 描く工具の長さ（工具長 `tool.cutting_length` を指定しない工具は、最も深い切削の位置で素材の上面 `stock_top` に
/// 届く長さ。ボールエンドミルの線は、球の中心から長さを測る）
fn drawn_tool_length(
    tool: &Tool<f64>,
    segments: &[(LineSegment3D<f64>, bool)],
    stock_top: f64,
) -> f64 {
    tool.cutting_length.unwrap_or_else(|| {
        let lowest_tip = segments
            .iter()
            .filter(|(_, is_cutting)| *is_cutting)
            .flat_map(|(segment, _)| [segment.start_point().z(), segment.end_point().z()])
            .fold(f64::INFINITY, f64::min);
        let ball_center = if tool.is_ball_end_mill() {
            tool.radius()
        } else {
            0.0
        };
        stock_top - lowest_tip - ball_center
    })
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
    use std::time::{Duration, Instant};

    use super::*;

    /// 表示中の製品形状（四角錐）を投入・実行・取り込みした切削アニメーション
    pub(super) fn animation() -> CuttingAnimation {
        crate::cutting_simulation_session::tests::imported_animation(None)
    }

    #[test]
    fn parts_of_a_block_hold_the_triangles_of_each_class() {
        let vertex = |x: f32| VertexData::new([x, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mesh = StockBlockMesh {
            block: [0, 0, 0],
            vertices: (0..5).map(|i| vertex(i as f32)).collect(),
            indices: vec![0, 1, 2, 2, 3, 4, 0, 2, 4],
            classes: vec![
                DeviationClass::Gouge,
                DeviationClass::NotCompared,
                DeviationClass::Gouge,
            ],
        };
        let triangles = |vertices: &[VertexData], indices: &[u32]| -> Vec<[f32; 3]> {
            indices
                .iter()
                .map(|&index| vertices[index as usize].position)
                .collect()
        };
        let expected = triangles(&mesh.vertices, &mesh.indices);
        let parts = mesh.into_parts();
        let classes = parts.each_ref().map(|(class, _, _)| *class);
        assert_eq!(
            classes,
            [
                DeviationClass::NotCompared,
                DeviationClass::Uncut,
                DeviationClass::Within,
                DeviationClass::Gouge,
            ]
        );
        // 分類ごとに、その分類の三角形を同じ順に持ち、使う頂点だけを持つ
        let [(_, plain_vertices, plain), (_, uncut_vertices, uncut), (_, within_vertices, within), (_, gouge_vertices, gouge)] =
            &parts;
        assert_eq!(triangles(plain_vertices, plain), expected[3..6]);
        assert_eq!(plain_vertices.len(), 3);
        assert!(uncut_vertices.is_empty() && uncut.is_empty());
        assert!(within_vertices.is_empty() && within.is_empty());
        assert_eq!(
            triangles(gouge_vertices, gouge),
            [&expected[0..3], &expected[6..9]].concat()
        );
        assert_eq!(gouge_vertices.len(), 4);
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
    fn turning_the_split_on_before_the_last_frame_arrives_splits_the_last_frame() {
        let mut animation = animation();
        let last = animation.frame_count() - 1;
        // 最後の時点を頼み、届く前に色分けを入にする（頼み直すのは、届いた時点ではなく頼んだ時点）
        animation.request_frame(last);
        animation.set_deviations_shown(true);
        let started = Instant::now();
        let mut classes = std::collections::HashSet::new();
        loop {
            if let Some(blocks) = animation.take_stock_update() {
                classes.extend(blocks.iter().flat_map(|mesh| mesh.classes.iter().copied()));
                if animation.frame() == last
                    && classes
                        .iter()
                        .any(|&class| class != DeviationClass::NotCompared)
                {
                    break;
                }
            }
            assert!(started.elapsed() < Duration::from_secs(60), "{classes:?}");
            std::thread::sleep(Duration::from_millis(1));
        }
        // 切ると、最後の時点の面を分けない面で作り直す
        animation.set_deviations_shown(false);
        let started = Instant::now();
        loop {
            if let Some(blocks) = animation.take_stock_update() {
                assert!(blocks.iter().all(|mesh| mesh
                    .classes
                    .iter()
                    .all(|&c| c == DeviationClass::NotCompared)));
                break;
            }
            assert!(started.elapsed() < Duration::from_secs(60));
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
    fn tools_without_a_length_reach_the_stock_top_from_the_deepest_cut() {
        use geo_algorithms::{LineSegment3DConstructor, Point3D};
        let line = |z0: f64, z1: f64| {
            LineSegment3D::new(Point3D::new(0.0, 0.0, z0), Point3D::new(10.0, 0.0, z1)).unwrap()
        };
        // 最も深い切削は 1.2（早送りの 0.5 は数えない）。素材の上面は 5.0
        let segments = [
            (line(3.0, 1.6), true),
            (line(1.6, 1.2), true),
            (line(0.5, 0.5), false),
        ];
        let mut ball = Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0);
        ball.cutting_length = None;
        // ボールエンドミルは球の中心（先端 + 半径 1.0）から測る
        assert!((drawn_tool_length(&ball, &segments, 5.0) - 2.8).abs() < 1e-12);
        let mut flat = Tool::flat_end_mill("FEM2".to_string(), 2.0, 10.0);
        flat.cutting_length = None;
        assert!((drawn_tool_length(&flat, &segments, 5.0) - 3.8).abs() < 1e-12);
        // 工具長を指定した工具は、その長さ
        flat.cutting_length = Some(7.0);
        assert_eq!(drawn_tool_length(&flat, &segments, 5.0), 7.0);
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
        let mut animation =
            crate::cutting_simulation_session::tests::imported_animation(Some(None));
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
    }
}
