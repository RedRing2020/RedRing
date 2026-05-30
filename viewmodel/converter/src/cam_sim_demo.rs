//! CAMシミュレーション向けのデバッグ/デモ専用データ生成。

use application::cam_orchestration::{
    ApplicationError, CamSimulationExecutionOrchestration, CamSimulationExecutionOrchestrator,
    CamSimulationExecutionRequest,
};
use cam_core::fixtures::{
    create_empty_toolpath, create_sample_ball_end_mill_tool, create_sample_flat_end_mill_tool,
    create_sample_toolpath,
};
use cam_core::{Tool, ToolPath};
use cam_sim::SnapshotInterval;
use geo_algorithms::{Aabb3D, Point3D};

/// CAMシミュレーションのデモシナリオ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamSimulationDemoScenario {
    Success,
    SuccessFlatEndMill,
    FailureEmptyToolpath,
}

/// CAMシミュレーションデモで使う ToolPath/Tool と付随メタデータ。
#[derive(Debug, Clone)]
pub struct CamSimulationDemoArtifacts {
    pub toolpath: ToolPath<f64>,
    pub tool: Tool<f64>,
    pub output_index: u32,
    pub local_key: String,
    pub description: Option<String>,
}

/// デモシナリオに応じた ToolPath/Tool と付随メタデータを組み立てる。
pub fn build_demo_artifacts_for_cam_simulation(
    scenario: CamSimulationDemoScenario,
) -> CamSimulationDemoArtifacts {
    let toolpath = match scenario {
        CamSimulationDemoScenario::Success | CamSimulationDemoScenario::SuccessFlatEndMill => {
            create_sample_toolpath()
        }
        CamSimulationDemoScenario::FailureEmptyToolpath => create_empty_toolpath(),
    };

    let tool = match scenario {
        CamSimulationDemoScenario::Success | CamSimulationDemoScenario::FailureEmptyToolpath => {
            create_sample_ball_end_mill_tool()
        }
        CamSimulationDemoScenario::SuccessFlatEndMill => create_sample_flat_end_mill_tool(),
    };

    let output_index = match scenario {
        CamSimulationDemoScenario::Success => 0,
        CamSimulationDemoScenario::FailureEmptyToolpath => 1,
        CamSimulationDemoScenario::SuccessFlatEndMill => 2,
    };

    CamSimulationDemoArtifacts {
        toolpath,
        tool,
        output_index,
        local_key: "sample_tool".to_string(),
        description: Some("CAM simulation sample tool".to_string()),
    }
}

/// デバッグ用：指定シナリオで CAM スナップショットサンプルを実行し export DTO を返す。
pub fn create_demo_snapshot_exports_for_scenario(
    scenario: CamSimulationDemoScenario,
) -> Result<application::cam_orchestration::CamSimulationExecutionResult, ApplicationError> {
    let demo = build_demo_artifacts_for_cam_simulation(scenario);
    let request = CamSimulationExecutionRequest {
        toolpath: demo.toolpath,
        tool: demo.tool,
        work_bounds: Aabb3D::new(
            Point3D::new(-60.0, -60.0, -20.0),
            Point3D::new(60.0, 60.0, 30.0),
        ),
        max_depth: 4,
        snapshot_interval: SnapshotInterval::default(),
    };

    CamSimulationExecutionOrchestrator.execute_simulation_snapshot_exports(request)
}

/// デバッグ用：CAMスナップショットサンプルを実行し、export DTO を返す。
pub fn create_sample_snapshot_exports_for_demo(
) -> Result<application::cam_orchestration::CamSimulationExecutionResult, ApplicationError> {
    create_demo_snapshot_exports_for_scenario(CamSimulationDemoScenario::Success)
}
