use cam_core::{Tool, ToolPath};
use geo_algorithms::octree::VoxelOctree;
use geo_algorithms::{LineSegment3D, Scalar};

use crate::error::SimulationError;

mod config;
mod engine;
mod segments;

/// スナップショット保存間隔の指定方法。
#[derive(Debug, Clone)]
pub enum SnapshotInterval {
    ByDistance {
        interval_mm: f64,
        include_segment_endpoints: bool,
    },
    ByAutoDistance {
        target_count: usize,
        min_interval_mm: f64,
        include_segment_endpoints: bool,
    },
}

impl Default for SnapshotInterval {
    fn default() -> Self {
        Self::ByDistance {
            interval_mm: 10.0,
            include_segment_endpoints: true,
        }
    }
}

/// 経路上の位置（どのセグメントのどこか）を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPosition {
    pub segment_index: usize,
    pub t: f64,
    pub accumulated_distance: f64,
}

/// 1時点のシミュレーション結果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshot<T: Scalar> {
    pub position: PathPosition,
    pub remaining_volume: T,
}

/// ViewModel連携用のスナップショット出力DTO（単位付きのf64へ正規化）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshotExport {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// 切削シミュレーション本体。
///
/// - 入力: ToolPath + Tool
/// - 出力: スナップショット列と更新済みVoxelOctree
#[derive(Debug, Clone)]
pub struct CuttingSimulator<T: Scalar> {
    voxel_tree: VoxelOctree<T>,
    interval: SnapshotInterval,
    snapshots: Vec<SimulationSnapshot<T>>,
    /// Arc セグメントを polyline 近似する際の弦誤差上限（mm）。
    arc_chord_tolerance_mm: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SnapshotConfig {
    pub(super) actual_interval_mm: f64,
    pub(super) include_endpoints: bool,
}

impl<T: Scalar> CuttingSimulator<T> {
    /// 新しいシミュレータを作成する。
    pub fn new(voxel_tree: VoxelOctree<T>, interval: SnapshotInterval) -> Self {
        Self {
            voxel_tree,
            interval,
            snapshots: Vec::new(),
            arc_chord_tolerance_mm: segments::DEFAULT_ARC_CHORD_TOLERANCE_MM,
        }
    }

    /// 内部VoxelOctreeへの参照を取得する。
    pub fn voxel_tree(&self) -> &VoxelOctree<T> {
        &self.voxel_tree
    }

    /// 内部VoxelOctreeへの可変参照を取得する。
    pub fn voxel_tree_mut(&mut self) -> &mut VoxelOctree<T> {
        &mut self.voxel_tree
    }

    /// 記録済みスナップショットを取得する。
    pub fn snapshots(&self) -> &[SimulationSnapshot<T>] {
        &self.snapshots
    }

    /// 記録済みスナップショットをViewModel連携向けDTOへ変換して返す。
    pub fn snapshot_exports_f64(&self) -> Vec<SimulationSnapshotExport> {
        self.snapshots
            .iter()
            .map(|snapshot| SimulationSnapshotExport {
                segment_index: snapshot.position.segment_index,
                segment_t: snapshot.position.t,
                accumulated_distance_mm: snapshot.position.accumulated_distance,
                remaining_volume_mm3: snapshot.remaining_volume.to_f64(),
            })
            .collect()
    }

    /// スナップショットをクリアする。
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    /// ToolPathをシミュレーションする（Phase 1a: フラットエンドミル限定）。
    pub fn simulate(
        &mut self,
        toolpath: &ToolPath<T>,
        tool: &Tool<T>,
    ) -> Result<(), SimulationError> {
        if !tool.is_flat_end_mill() {
            return Err(SimulationError::UnsupportedToolType);
        }

        let segments = self.collect_line_segments(toolpath)?;
        if segments.is_empty() {
            return Err(SimulationError::EmptyToolpath);
        }

        let config = self.calculate_snapshot_config(
            segments
                .iter()
                .map(|(segment, _)| self.segment_length(segment))
                .sum(),
        )?;

        self.simulate_segments_with_flags(&segments, tool.radius(), config);
        Ok(())
    }

    /// 線分列を直接シミュレーションする補助API。
    pub fn simulate_line_segments(&mut self, segments: &[LineSegment3D<T>], tool_radius: T) {
        let config = self
            .calculate_snapshot_config(
                segments
                    .iter()
                    .map(|segment| self.segment_length(segment))
                    .sum(),
            )
            .unwrap_or(SnapshotConfig {
                actual_interval_mm: 10.0,
                include_endpoints: true,
            });

        let flagged: Vec<(LineSegment3D<T>, bool)> = segments
            .iter()
            .cloned()
            .map(|segment| (segment, true))
            .collect();
        self.simulate_segments_with_flags(&flagged, tool_radius, config);
    }
}
