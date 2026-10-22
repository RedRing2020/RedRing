use geo_algorithms::{
    Aabb3D, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, LineSegment3D, LineSegment3DProperties,
};

use cam_core::{Tool, ToolPath};

use crate::constants::TRI_DEXEL_DISPLAY_LINES;
use crate::error::SimulationError;
use crate::simulation_artifact::CuttingSimulationArtifact;
use crate::stock_model::StockModel;
use crate::tri_dexel::TriDexelModel;

mod config;
mod engine;
mod segments;

pub use segments::{
    collect_toolpath_line_segments, collect_toolpath_line_segments_with_arc_options,
};

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

impl SnapshotInterval {
    /// 間隔の指定が有効か（間隔・最小の間隔が正の有限値、目標の記録数が 1 以上）
    pub(crate) fn is_valid(&self) -> bool {
        match *self {
            Self::ByDistance { interval_mm, .. } => interval_mm.is_finite() && interval_mm > 0.0,
            Self::ByAutoDistance {
                target_count,
                min_interval_mm,
                ..
            } => target_count > 0 && min_interval_mm.is_finite() && min_interval_mm > 0.0,
        }
    }
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
pub struct SimulationSnapshot {
    pub position: PathPosition,
    /// その区間までの切削を終えた時点の残りの体積
    pub remaining_volume: f64,
}

/// ViewModel連携用のスナップショット出力DTO（単位付きのf64へ正規化）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshotExport {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// 切削シミュレーション本体（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md）。
///
/// - 入力: 素材（直方体）・ToolPath・Tool
/// - 素材の正本（`StockModel`）と表示用のトリデクセル（`TriDexelModel`）を作り、
///   経路の距離に沿ったスナップショットの時点と残りの体積を記録する
#[derive(Debug, Clone)]
pub struct CuttingSimulator {
    stock: Aabb3D<f64>,
    interval: SnapshotInterval,
    snapshots: Vec<SimulationSnapshot>,
    /// 各スナップショットの時点までに差し引いた掃引体（切削の区間）の数
    pub(crate) snapshot_applied: Vec<usize>,
    /// 最後に実行した結果の表示用のトリデクセル
    pub(crate) dexels: Option<TriDexelModel>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SnapshotConfig {
    pub(super) actual_interval_mm: f64,
    pub(super) include_endpoints: bool,
}

impl CuttingSimulator {
    /// 新しいシミュレータを作成する。
    pub fn new(stock: Aabb3D<f64>, interval: SnapshotInterval) -> Self {
        Self {
            stock,
            interval,
            snapshots: Vec::new(),
            snapshot_applied: Vec::new(),
            dexels: None,
        }
    }

    /// 記録済みスナップショットを取得する。
    pub fn snapshots(&self) -> &[SimulationSnapshot] {
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
                remaining_volume_mm3: snapshot.remaining_volume,
            })
            .collect()
    }

    /// 最後に実行した結果の成果物の中身（実行していない場合は `None`）
    pub fn artifact(&self) -> Option<CuttingSimulationArtifact> {
        let dexels = self.dexels.as_ref()?;
        Some(CuttingSimulationArtifact {
            snapshots: self.snapshot_exports_f64(),
            grids: dexels.layouts(),
            snapshot_changes: dexels.snapshot_changes(&self.snapshot_applied),
        })
    }

    /// ToolPathをシミュレーションする。
    ///
    /// 円弧は既定の弦の許容誤差で直線の区間の列にする。表示用のトリデクセルは、素材の最大の辺を
    /// `TRI_DEXEL_DISPLAY_LINES` 本の格子線に分ける。
    ///
    /// # Errors
    ///
    /// 経路が空の場合・工具がフラット・ボールエンドミルでない場合・記録の間隔が不正な場合。
    pub fn simulate(
        &mut self,
        toolpath: &ToolPath<f64>,
        tool: &Tool<f64>,
    ) -> Result<(), SimulationError> {
        let segments =
            collect_toolpath_line_segments(toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);
        if segments.is_empty() {
            return Err(SimulationError::EmptyToolpath);
        }
        let model = StockModel::from_segments(self.stock, &segments, tool)?;
        let config = self.calculate_snapshot_config(
            segments
                .iter()
                .map(|(segment, _)| segment_length(segment))
                .sum(),
        )?;
        let max_extent = self
            .stock
            .width()
            .max(self.stock.height())
            .max(self.stock.depth());
        let dexels =
            TriDexelModel::from_stock_model(&model, max_extent / TRI_DEXEL_DISPLAY_LINES as f64);
        self.record_snapshots(&segments, &dexels, config);
        self.dexels = Some(dexels);
        Ok(())
    }
}

/// 線分の長さ
fn segment_length(segment: &LineSegment3D<f64>) -> f64 {
    segment.start_point().distance_to(segment.end_point())
}
