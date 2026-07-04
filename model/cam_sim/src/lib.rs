//! CAMシミュレーション実行クレート
//!
//! CAMドメインのシミュレーション実行責務を提供します。
//! 幾何計算カーネルは `geo_algorithms` に委譲します。
//! Phase 1a では 3軸固定・フラットエンドミル限定の最小実装を提供します。

mod error;
mod exact_work;
mod job_adapter;
mod simulator;
mod workflow;

/// シミュレーション実行時のエラー型。
pub use error::SimulationError;
/// SAT-Cut PoC向けのExactWork trait定義。
pub use exact_work::{
    ExactToolPrimitive, ExactWorkModel, ExactWorkProjectionCache, PrimitiveSetExactWork,
};
/// Job Manager接続用アダプタ。
pub use job_adapter::CamJobExecutorAdapter;
/// シミュレーション実行APIとスナップショット関連型。
pub use simulator::{
    CuttingSimulator, PathPosition, SimulationSnapshot, SimulationSnapshotExport, SnapshotInterval,
    collect_toolpath_line_segments, collect_toolpath_line_segments_with_arc_options,
};
/// CAM工程向けワークフロー制約ファサード。
pub use workflow::{CamWorkflowError, CamWorkflowSubmitter};

#[cfg(test)]
mod tests;
