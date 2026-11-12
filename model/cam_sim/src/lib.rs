//! CAMシミュレーション実行クレート
//!
//! CAMドメインのシミュレーション実行責務を提供します。
//! 幾何計算カーネルは `geo_algorithms` に委譲します。
//! 3 軸固定で、フラット・ボールエンドミルを扱います。
//! 素材の正本は工具の掃引プリミティブの列で表します（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md）。

mod binary_codec;
mod constants;
mod dexel_normal;
mod dexel_surface;
mod error;
mod job_adapter;
mod job_input_file;
mod job_run;
mod preflight;
mod product_deviation;
mod replay;
mod simulation_artifact;
mod simulation_input;
mod simulator;
mod solver_input;
mod stock_model;
mod tri_dexel;
mod workflow;

/// デクセルの区間の端の、量子化した法線。
pub use dexel_normal::DexelNormal;
/// シミュレーション実行時のエラー型。
/// トリデクセルの表示用の面。
pub use dexel_surface::{DeviationClass, DexelSurface, SurfaceDeviations, SurfaceMesh};
pub use error::SimulationError;
/// Job Manager接続用アダプタ。
pub use job_adapter::CamJobExecutorAdapter;
/// ジョブの入力のファイル（ツールパス生成の入力・切削シミュレーションの入力）の書き出し・読み込み。
pub use job_input_file::{
    read_cam_solver_input_v1, read_cutting_simulation_input_v1, write_cam_solver_input_v1,
    write_cutting_simulation_input_v1,
};
/// ジョブ 1 つの計算（入力から成果物のバイト列を作る）。
pub use job_run::{
    CuttingSimulationRun, JobRunFailure, TOOLPATH_MEMORY_LIMIT_BYTES, run_cam_solver,
    run_cutting_simulation, toolpath_memory_bytes,
};
pub use preflight::{CuttingSimulationInputError, cutting_simulation_input_errors};
pub use product_deviation::ProductDeviations;
/// 成果物の差分のキーフレームと正本から、任意の時点の素材を作り直す再生の入口。
pub use replay::CuttingSimulationReplay;
/// 切削シミュレーションの成果物（中身の型と書き出し・読み込み）。
pub use simulation_artifact::{
    CuttingSimulationArtifact, DexelChanges, DexelGridLayout, DexelGridStates, DexelState,
    read_cutting_simulation_artifact_v1, write_cutting_simulation_artifact_v1,
};
/// CuttingSimulation の InputRef -> 素材・工具・精度の設定の解決境界。
pub use simulation_input::{
    CuttingSimulationInput, CuttingSimulationInputProvider, InMemoryCuttingSimulationInputStore,
    stock_around_product,
};
/// シミュレーション実行APIとスナップショット関連型。
pub use simulator::{
    CuttingSimulator, PathPosition, SimulationSnapshot, SimulationSnapshotExport, SnapshotInterval,
    collect_toolpath_line_segments, collect_toolpath_line_segments_with_arc_options,
};
/// CamProcess の InputRef -> solver 入力解決境界。
pub use solver_input::{
    CAM_INPUT_DOMAIN, CamSolverInputProvider, InMemoryCamSolverInputStore, validate_cam_input_ref,
};
/// 素材の正本（初期素材と工具の掃引プリミティブの列）。
pub use stock_model::StockModel;
/// 素材の派生表現のトリデクセル（変化の履歴つき）。
pub use tri_dexel::TriDexelModel;
/// CAM工程向けワークフロー制約ファサード。
pub use workflow::{CamWorkflowError, CamWorkflowSubmitter};

#[cfg(test)]
mod solver_job_tests;
#[cfg(test)]
mod tests;
