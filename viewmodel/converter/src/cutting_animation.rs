//! cutting_animation - 切削シミュレーションの成果物から、切削アニメーションの描画用のデータを作る
//!
//! 今の時点の素材の面（ブロックごと）を 1 つだけ持ち、時点を進める・戻す場合は、区間が変わった部分のブロック
//! だけを作り直して返す。工具の形とツールパスの線は、ツールパスの成果物と工具から作る。

use cam_core::{read_toolpath_artifact_v1, BinaryFormatError, Tool, ToolPath};
use cam_sim::{
    collect_toolpath_line_segments, read_cutting_simulation_artifact_v1, CuttingSimulationArtifact,
    DexelChange, DexelSurface, SurfaceMesh,
};
use geo_algorithms::{LineSegment3D, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM};

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

/// 切削アニメーションを作れない理由
#[derive(Debug)]
pub enum CuttingAnimationError {
    /// 成果物を読めない
    Artifact(BinaryFormatError),
    /// スナップショットがない
    NoSnapshot,
}

impl std::fmt::Display for CuttingAnimationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Artifact(error) => write!(f, "failed to read the artifact: {error}"),
            Self::NoSnapshot => write!(f, "the cutting simulation has no snapshot"),
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
#[derive(Debug, Clone)]
pub struct CuttingAnimation {
    artifact: CuttingSimulationArtifact,
    /// 時点ごとの、前の時点へ戻す変化（`artifact.snapshot_changes` と同じ並び）
    reverse_changes: Vec<Vec<DexelChange>>,
    surface: DexelSurface,
    frame: usize,
    toolpath: ToolPath<f64>,
    /// 切削シミュレーションと同じ分け方の、ツールパスの直線の区間（スナップショットの区間の番号の対象）
    segments: Vec<(LineSegment3D<f64>, bool)>,
    tool: Tool<f64>,
}

impl CuttingAnimation {
    /// ツールパスと切削シミュレーションの成果物と工具から、最初の時点の切削アニメーションを作る
    ///
    /// # Errors
    ///
    /// 成果物を読めない場合・スナップショットがない場合。
    pub fn new(
        toolpath_artifact: &[u8],
        simulation_artifact: &[u8],
        tool: Tool<f64>,
    ) -> Result<Self, CuttingAnimationError> {
        let (_, toolpath) = read_toolpath_artifact_v1(&mut &toolpath_artifact[..])?;
        let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &simulation_artifact[..])?;
        if artifact.snapshots.is_empty() {
            return Err(CuttingAnimationError::NoSnapshot);
        }
        let segments =
            collect_toolpath_line_segments(&toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);
        Ok(Self {
            surface: DexelSurface::at_snapshot(&artifact, 0),
            reverse_changes: artifact.reverse_snapshot_changes(),
            artifact,
            frame: 0,
            toolpath,
            segments,
            tool,
        })
    }

    /// 時点（スナップショット）の数
    pub fn frame_count(&self) -> usize {
        self.artifact.snapshots.len()
    }

    /// 今の時点
    pub fn frame(&self) -> usize {
        self.frame
    }

    /// 各時点の位置と統計の系列
    pub fn snapshot_series(
        &self,
        source: impl Into<String>,
    ) -> DomainSnapshotSeries<CamSimulationSnapshotInput> {
        let inputs = cam_snapshot_exports_to_inputs(&self.artifact.snapshots);
        cam_snapshot_inputs_to_domain_series(source, &inputs)
    }

    /// 今の時点の、すべてのブロックの素材の面（三角形のないブロックは含めない）
    pub fn stock_blocks(&self) -> Vec<StockBlockMesh> {
        let [bx, by, bz] = self.surface.block_counts();
        let blocks: Vec<[usize; 3]> = (0..bx)
            .flat_map(|x| (0..by).flat_map(move |y| (0..bz).map(move |z| [x, y, z])))
            .collect();
        self.block_meshes(&blocks)
            .into_iter()
            .filter(|mesh| !mesh.indices.is_empty())
            .collect()
    }

    /// 時点 `frame` へ移し、面が変わりうるブロックの、移った後の素材の面を返す（範囲外は最後の時点にする）
    ///
    /// 進む場合はその間の変化を、戻る場合はその間の戻す変化を当て、区間が変わった部分のブロックだけを作り直す。
    /// 三角形がなくなったブロックは、三角形のない面として返す。
    pub fn seek(&mut self, frame: usize) -> Vec<StockBlockMesh> {
        let target = frame.min(self.frame_count() - 1);
        let mut dirty = std::collections::BTreeSet::new();
        if target < self.frame {
            for snapshot in ((target + 1)..=self.frame).rev() {
                dirty.extend(self.surface.apply_changes(&self.reverse_changes[snapshot]));
            }
        } else {
            for snapshot in (self.frame + 1)..=target {
                dirty.extend(
                    self.surface
                        .apply_changes(&self.artifact.snapshot_changes[snapshot]),
                );
            }
        }
        self.frame = target;
        let blocks: Vec<[usize; 3]> = dirty.into_iter().collect();
        self.block_meshes(&blocks)
    }

    /// 今の時点の工具の形（工具の先端の位置。位置が分からない場合は空）
    pub fn tool_wireframe(
        &self,
        settings: &ToolWireframeVisualizationSettings,
    ) -> Vec<WireframeVertex> {
        let snapshot = &self.artifact.snapshots[self.frame];
        let Some(tip) =
            tool_tip_position(&self.segments, snapshot.segment_index, snapshot.segment_t)
        else {
            return Vec::new();
        };
        let radius = self.tool.radius();
        let is_ball = self.tool.is_ball_end_mill();
        // 工具長を指定しない工具は、素材の上面の高さまで描く（ボールは球の中心から測る）
        let length = self.tool.cutting_length.unwrap_or_else(|| {
            let stock_top = self.artifact.grids[2].initial.1;
            stock_top - tip.z() - if is_ball { radius } else { 0.0 }
        });
        if is_ball {
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

    /// ブロックの列 `blocks` の素材の面（複数のコアで並べて作る）
    fn block_meshes(&self, blocks: &[[usize; 3]]) -> Vec<StockBlockMesh> {
        let to_f32 = |value: [f64; 3]| value.map(|component| component as f32);
        blocks
            .iter()
            .zip(self.surface.mesh_blocks(blocks))
            .map(
                |(
                    &block,
                    SurfaceMesh {
                        positions,
                        normals,
                        indices,
                    },
                )| StockBlockMesh {
                    block,
                    vertices: positions
                        .into_iter()
                        .zip(normals)
                        .map(|(position, normal)| VertexData::new(to_f32(position), to_f32(normal)))
                        .collect(),
                    indices,
                },
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::OnceLock;

    use cam_core::{
        write_toolpath_payload_v1, ArtifactHeaderV1, ArtifactKind, ContourLevelPath,
        CoordinateFrame, CuttingDirection, LengthUnit, PathSegment, SegmentType,
    };
    use cam_sim::{write_cutting_simulation_artifact_v1, CuttingSimulator, SnapshotInterval};
    use geo_algorithms::{Aabb3D, Point3D};

    use super::*;

    fn tool() -> Tool<f64> {
        Tool::ball_end_mill("BEM2".to_string(), 2.0, 10.0)
    }

    /// 素材の上面を 3 本の往復の線で削るツールパスと切削シミュレーションの成果物（テストの間で 1 回だけ作る）
    fn artifacts() -> &'static (Vec<u8>, Vec<u8>) {
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
            simulator.simulate(&toolpath, &tool()).unwrap();
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
        CuttingAnimation::new(toolpath, simulation, tool()).unwrap()
    }

    /// ブロックごとの面を、頂点の位置と三角形で比べられる形にする
    fn positions(
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

    fn by_block(blocks: Vec<StockBlockMesh>) -> HashMap<[usize; 3], StockBlockMesh> {
        blocks.into_iter().map(|mesh| (mesh.block, mesh)).collect()
    }

    #[test]
    fn frames_follow_the_simulation_snapshots() {
        let animation = animation();
        assert!(animation.frame_count() > 2);
        assert_eq!(animation.frame(), 0);
        assert_eq!(
            animation.snapshot_series("cam_sim").frames.len(),
            animation.frame_count()
        );
        assert!(!animation.stock_blocks().is_empty());
        assert!(!animation
            .toolpath_wireframe(&ToolPathVisualizationSettings::default())
            .is_empty());
    }

    #[test]
    fn updating_changed_blocks_forward_and_backward_matches_the_state_at_each_frame() {
        let mut animation = animation();
        let mut shown = by_block(animation.stock_blocks());
        let last = animation.frame_count() - 1;
        for frame in [1, last / 2, last, last / 2, 1, 0, last] {
            for mesh in animation.seek(frame) {
                shown.insert(mesh.block, mesh);
            }
            assert_eq!(animation.frame(), frame);
            assert_eq!(
                positions(&shown),
                positions(&by_block(animation.stock_blocks())),
                "frame {frame}"
            );
        }
    }

    #[test]
    fn tool_wireframe_is_placed_at_the_snapshot_position() {
        let mut animation = animation();
        let settings = ToolWireframeVisualizationSettings::default();
        for frame in 0..animation.frame_count() {
            animation.seek(frame);
            let snapshot = animation.artifact.snapshots[frame];
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

    #[test]
    fn broken_artifacts_are_rejected() {
        let (toolpath, simulation) = artifacts();
        let truncated = &simulation[..simulation.len() - 1];
        assert!(matches!(
            CuttingAnimation::new(toolpath, truncated, tool()),
            Err(CuttingAnimationError::Artifact(_))
        ));
        assert!(matches!(
            CuttingAnimation::new(simulation, simulation, tool()),
            Err(CuttingAnimationError::Artifact(_))
        ));
    }
}
