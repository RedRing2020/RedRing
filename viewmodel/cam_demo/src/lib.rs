//! CAMシミュレーションのデモ導線専用クレート。
//! 本番向け変換ロジックとは分離して、debug/demo専用の入力生成と実行を扱う。

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
use converter::cam_sim_visualization_converter::{
    build_cam_simulation_visualization_bundle_with_tool_settings,
    CamSimulationVisualizationBuildInput, CamSimulationVisualizationBundle,
    CamSimulationVisualizationError, ToolWireframeVisualizationSettings,
};
use converter::octree_converter::OctreeVisualizationSettings;
use converter::snapshot_converter::{
    cam_snapshot_exports_to_inputs, cam_snapshot_inputs_to_domain_series,
    CamSimulationSnapshotInput, DomainSnapshotSeries,
};
use converter::toolpath_converter::ToolPathVisualizationSettings;
use geo_algorithms::{Aabb3D, Point3D};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamSimulationDemoScenario {
    Success,
    SuccessFlatEndMill,
    FailureEmptyToolpath,
}

#[derive(Debug, Clone)]
pub struct CamSimulationDemoArtifacts {
    pub toolpath: ToolPath<f64>,
    pub tool: Tool<f64>,
    pub output_index: u32,
    pub local_key: String,
    pub description: Option<String>,
}

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

pub fn load_demo_cam_snapshot_domain_series(
) -> Result<DomainSnapshotSeries<CamSimulationSnapshotInput>, ApplicationError> {
    let exports =
        create_demo_snapshot_exports_for_scenario(CamSimulationDemoScenario::Success)?.exports;
    let inputs = cam_snapshot_exports_to_inputs(&exports);

    Ok(cam_snapshot_inputs_to_domain_series("cam_sim", &inputs))
}

pub fn build_demo_cam_simulation_visualization_bundle_with_tool_settings(
    settings: &OctreeVisualizationSettings,
    tool_wireframe_settings: &ToolWireframeVisualizationSettings,
    toolpath_settings: &ToolPathVisualizationSettings,
    scenario: CamSimulationDemoScenario,
) -> Result<CamSimulationVisualizationBundle, CamSimulationVisualizationError> {
    let demo = build_demo_artifacts_for_cam_simulation(scenario);

    let input = CamSimulationVisualizationBuildInput {
        toolpath: demo.toolpath,
        tool: demo.tool,
        feature_id: "cam_sim_visualization".to_string(),
        output_index: demo.output_index,
        local_key: demo.local_key,
        description: demo.description,
        snapshot_source: "cam_sim".to_string(),
    };

    build_cam_simulation_visualization_bundle_with_tool_settings(
        settings,
        tool_wireframe_settings,
        toolpath_settings,
        input,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cam_core::ToolType;
    use geo_algorithms::octree::OctreeTolerance;

    fn default_octree_settings() -> OctreeVisualizationSettings {
        OctreeVisualizationSettings {
            max_depth: 3,
            gradient_start: [0.2, 1.0, 1.0],
            gradient_end: [1.0, 0.4, 0.4],
            octree_tolerance: OctreeTolerance::default(),
        }
    }

    #[test]
    fn test_build_demo_artifacts_success_scenario_mappings() {
        let success = build_demo_artifacts_for_cam_simulation(CamSimulationDemoScenario::Success);
        let success_flat =
            build_demo_artifacts_for_cam_simulation(CamSimulationDemoScenario::SuccessFlatEndMill);
        let failure_empty = build_demo_artifacts_for_cam_simulation(
            CamSimulationDemoScenario::FailureEmptyToolpath,
        );

        assert_eq!(success.output_index, 0);
        assert_eq!(success.tool.tool_type, ToolType::BallEndMill);
        assert!(success.toolpath.total_segments() > 0);

        assert_eq!(success_flat.output_index, 2);
        assert_eq!(success_flat.tool.tool_type, ToolType::FlatEndMill);
        assert!(success_flat.toolpath.total_segments() > 0);

        assert_eq!(failure_empty.output_index, 1);
        assert_eq!(failure_empty.tool.tool_type, ToolType::BallEndMill);
        assert_eq!(failure_empty.toolpath.total_segments(), 0);
    }

    #[test]
    fn test_build_demo_cam_simulation_visualization_bundle_failure_empty_toolpath() {
        let result = build_demo_cam_simulation_visualization_bundle_with_tool_settings(
            &default_octree_settings(),
            &ToolWireframeVisualizationSettings::default(),
            &ToolPathVisualizationSettings::default(),
            CamSimulationDemoScenario::FailureEmptyToolpath,
        );

        assert!(matches!(
            result,
            Err(CamSimulationVisualizationError::Application(
                ApplicationError::Simulation(_)
            ))
        ));
    }
}
