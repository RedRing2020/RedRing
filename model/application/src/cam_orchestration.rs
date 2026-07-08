//! CAM向け orchestration 境界。

use cam_core::{Tool, ToolPath};
use cam_sim::{CuttingSimulator, SimulationError, SimulationSnapshotExport, SnapshotInterval};
use geo_algorithms::{Aabb3D, LineSegment3D, Point3D, octree::VoxelOctree};

/// Application境界で返却する統一エラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    Simulation(String),
    ToolEntity(String),
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simulation(message) => write!(f, "{}", message),
            Self::ToolEntity(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for ApplicationError {}

impl From<SimulationError> for ApplicationError {
    fn from(value: SimulationError) -> Self {
        Self::Simulation(value.to_string())
    }
}

/// CAMシミュレーション由来の最小スナップショットDTO。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamSnapshotFrame {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// CAMスナップショット系列生成の入力境界。
#[derive(Debug, Clone, PartialEq)]
pub struct CamSnapshotSeriesRequest {
    pub source: String,
    pub frames: Vec<CamSnapshotFrame>,
}

/// CAMスナップショット系列生成の出力境界。
#[derive(Debug, Clone, PartialEq)]
pub struct CamSnapshotSeriesResult {
    pub frame_count: usize,
    pub source: String,
    pub frames: Vec<CamSnapshotFrame>,
}

/// CAMスナップショット関連ユースケースの orchestration port。
pub trait CamSnapshotSeriesOrchestration {
    fn create_snapshot_series(
        &self,
        request: CamSnapshotSeriesRequest,
    ) -> Result<CamSnapshotSeriesResult, String>;
}

/// スナップショット系列 orchestration の既定実装。
#[derive(Debug, Default, Clone, Copy)]
pub struct CamSnapshotSeriesOrchestrator;

impl CamSnapshotSeriesOrchestration for CamSnapshotSeriesOrchestrator {
    fn create_snapshot_series(
        &self,
        request: CamSnapshotSeriesRequest,
    ) -> Result<CamSnapshotSeriesResult, String> {
        Ok(CamSnapshotSeriesResult {
            frame_count: request.frames.len(),
            source: request.source,
            frames: request.frames,
        })
    }
}

/// cam_sim の export DTO を Application境界DTOに正規化する。
pub fn snapshot_exports_to_frames(exports: &[SimulationSnapshotExport]) -> Vec<CamSnapshotFrame> {
    exports
        .iter()
        .map(|export| CamSnapshotFrame {
            segment_index: export.segment_index,
            segment_t: export.segment_t,
            accumulated_distance_mm: export.accumulated_distance_mm,
            remaining_volume_mm3: export.remaining_volume_mm3,
        })
        .collect()
}

/// cam_sim export DTO から境界シリーズを組み立てる。
pub fn create_snapshot_series_from_exports(
    source: impl Into<String>,
    exports: &[SimulationSnapshotExport],
) -> Result<CamSnapshotSeriesResult, String> {
    CamSnapshotSeriesOrchestrator.create_snapshot_series(CamSnapshotSeriesRequest {
        source: source.into(),
        frames: snapshot_exports_to_frames(exports),
    })
}

/// cam_sim スナップショット実行の入力境界。
#[derive(Debug, Clone)]
pub struct CamSimulationExecutionRequest {
    pub toolpath: ToolPath<f64>,
    pub tool: Tool<f64>,
    pub work_bounds: Aabb3D<f64>,
    pub max_depth: usize,
    pub snapshot_interval: SnapshotInterval,
}

/// cam_sim スナップショット実行の出力境界。
#[derive(Debug, Clone, PartialEq)]
pub struct CamSimulationExecutionResult {
    pub exports: Vec<SimulationSnapshotExport>,
}

/// CAMシミュレーション実行ユースケースの orchestration port。
pub trait CamSimulationExecutionOrchestration {
    fn execute_simulation_snapshot_exports(
        &self,
        request: CamSimulationExecutionRequest,
    ) -> Result<CamSimulationExecutionResult, ApplicationError>;
}

/// CAMシミュレーション実行 orchestration の既定実装。
#[derive(Debug, Default, Clone, Copy)]
pub struct CamSimulationExecutionOrchestrator;

impl CamSimulationExecutionOrchestration for CamSimulationExecutionOrchestrator {
    fn execute_simulation_snapshot_exports(
        &self,
        request: CamSimulationExecutionRequest,
    ) -> Result<CamSimulationExecutionResult, ApplicationError> {
        execute_simulation_snapshot_exports(request).map_err(ApplicationError::from)
    }
}

/// cam_sim を実行し、スナップショット出力を Application 境界結果として返す。
pub fn execute_simulation_snapshot_exports(
    request: CamSimulationExecutionRequest,
) -> Result<CamSimulationExecutionResult, SimulationError> {
    let voxel_tree = VoxelOctree::new(request.work_bounds, request.max_depth);
    let mut simulator = CuttingSimulator::new(voxel_tree, request.snapshot_interval);
    simulator.simulate(&request.toolpath, &request.tool)?;

    Ok(CamSimulationExecutionResult {
        exports: simulator.snapshot_exports_f64(),
    })
}

/// Tool を Entity ストレージに追加するための入力DTO。
#[derive(Debug, Clone)]
pub struct AddToolToEntityStorageRequest {
    pub tool: Tool<f64>,
    pub feature_id: String,
    pub output_index: u32,
    pub local_key: String,
    pub description: Option<String>,
}

/// Tool を Entity ストレージに追加した結果のDTO。
#[derive(Debug, Clone)]
pub struct AddToolToEntityStorageResult {
    pub entity_id: String,
    pub tool_name: String,
    pub description: Option<String>,
}

/// Tool エンティティ管理ユースケースの orchestration port。
pub trait ToolEntityManagementOrchestration {
    fn add_tool_to_storage(
        &self,
        request: AddToolToEntityStorageRequest,
    ) -> Result<AddToolToEntityStorageResult, ApplicationError>;
}

/// Tool エンティティ管理の既定実装。
#[derive(Debug, Default, Clone, Copy)]
pub struct ToolEntityManagementOrchestrator;

impl ToolEntityManagementOrchestration for ToolEntityManagementOrchestrator {
    fn add_tool_to_storage(
        &self,
        request: AddToolToEntityStorageRequest,
    ) -> Result<AddToolToEntityStorageResult, ApplicationError> {
        let tool_name = request.tool.id.clone();
        let entity_id = format!(
            "redring.entity:{}:{}:{}:{}",
            request.feature_id, request.output_index, request.local_key, tool_name
        );

        Ok(AddToolToEntityStorageResult {
            entity_id,
            tool_name,
            description: request.description,
        })
    }
}

/// ToolPath セグメント列から、シミュレーション用ワーク境界 AABB を推定する。
///
/// セグメントが空の場合はデフォルト境界（X/Y ±60mm、Z -20..5mm）を返す。
pub fn compute_toolpath_work_bounds(
    segments: &[(LineSegment3D<f64>, bool)],
    tool_radius: f64,
) -> Aabb3D<f64> {
    if segments.is_empty() {
        return Aabb3D::new(
            Point3D::new(-60.0, -60.0, -20.0),
            Point3D::new(60.0, 60.0, 5.0),
        );
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut min_z = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut max_z = f64::NEG_INFINITY;
    let mut cutting_min_z = f64::INFINITY;
    let mut cutting_max_z = f64::NEG_INFINITY;

    for (segment, is_cutting) in segments {
        let points = [segment.start(), segment.end()];
        for point in points {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            min_z = min_z.min(point.z());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
            max_z = max_z.max(point.z());
            if *is_cutting {
                cutting_min_z = cutting_min_z.min(point.z());
                cutting_max_z = cutting_max_z.max(point.z());
            }
        }
    }

    if !cutting_min_z.is_finite() || !cutting_max_z.is_finite() {
        cutting_min_z = min_z;
        cutting_max_z = max_z;
    }

    let xy_margin = (tool_radius * 3.0).max(10.0);
    let z_top_margin = (tool_radius * 0.2).max(0.5);
    let z_bottom_margin = (tool_radius * 4.0).max(10.0);

    Aabb3D::new(
        Point3D::new(
            min_x - xy_margin,
            min_y - xy_margin,
            cutting_min_z - z_bottom_margin,
        ),
        Point3D::new(
            max_x + xy_margin,
            max_y + xy_margin,
            cutting_max_z + z_top_margin,
        ),
    )
}

/// work_bounds 内に入り込む非切削セグメント数を数える（干渉確認用）。
pub fn count_non_cutting_interference_segments(
    segments: &[(LineSegment3D<f64>, bool)],
    work_bounds: &Aabb3D<f64>,
    tool_radius: f64,
) -> usize {
    let stock_min = work_bounds.min();
    let stock_max = work_bounds.max();

    segments
        .iter()
        .filter(|(_, is_cutting)| !*is_cutting)
        .filter(|(segment, _)| {
            let seg_min_x = segment.start().x().min(segment.end().x()) - tool_radius;
            let seg_min_y = segment.start().y().min(segment.end().y()) - tool_radius;
            let seg_min_z = segment.start().z().min(segment.end().z());

            let seg_max_x = segment.start().x().max(segment.end().x()) + tool_radius;
            let seg_max_y = segment.start().y().max(segment.end().y()) + tool_radius;
            let seg_max_z = segment.start().z().max(segment.end().z()) + 0.0001;

            !(seg_max_x < stock_min.x()
                || seg_min_x > stock_max.x()
                || seg_max_y < stock_min.y()
                || seg_min_y > stock_max.y()
                || seg_max_z < stock_min.z()
                || seg_min_z > stock_max.z())
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cam_core::CuttingDirection;

    mod demo_symbol_guard {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../test_helpers/demo_symbol_guard.rs"
        ));
    }

    #[test]
    fn test_create_snapshot_series_from_frames() {
        let request = CamSnapshotSeriesRequest {
            source: "cam_sim".to_string(),
            frames: vec![CamSnapshotFrame {
                segment_index: 1,
                segment_t: 0.5,
                accumulated_distance_mm: 12.0,
                remaining_volume_mm3: 980.0,
            }],
        };

        let result = CamSnapshotSeriesOrchestrator
            .create_snapshot_series(request)
            .expect("series creation should succeed");

        assert_eq!(result.frame_count, 1);
        assert_eq!(result.source, "cam_sim");
        assert_eq!(result.frames[0].segment_index, 1);
    }

    #[test]
    fn test_snapshot_exports_to_frames() {
        let exports = vec![SimulationSnapshotExport {
            segment_index: 2,
            segment_t: 0.75,
            accumulated_distance_mm: 33.0,
            remaining_volume_mm3: 812.0,
        }];

        let result = create_snapshot_series_from_exports("cam_sim", &exports)
            .expect("series creation from exports should succeed");

        assert_eq!(result.frame_count, 1);
        assert_eq!(result.frames[0].segment_index, 2);
        assert_eq!(result.frames[0].segment_t, 0.75);
    }

    #[test]
    fn test_execute_simulation_snapshot_exports() {
        let request = CamSimulationExecutionRequest {
            toolpath: ToolPath::new(
                "endmill_3mm".to_string(),
                CuttingDirection::Down,
                vec![],
                vec![],
                vec![],
            ),
            tool: Tool::flat_end_mill("endmill_3mm".to_string(), 10.0, 50.0),
            work_bounds: Aabb3D::new(
                Point3D::new(-60.0, -60.0, -20.0),
                Point3D::new(60.0, 60.0, 30.0),
            ),
            max_depth: 4,
            snapshot_interval: SnapshotInterval::default(),
        };

        let result = execute_simulation_snapshot_exports(request);
        assert!(matches!(result, Err(SimulationError::EmptyToolpath)));
    }

    #[test]
    fn test_cam_simulation_execution_orchestrator_normalizes_error() {
        let request = CamSimulationExecutionRequest {
            toolpath: ToolPath::new(
                "endmill_3mm".to_string(),
                CuttingDirection::Down,
                vec![],
                vec![],
                vec![],
            ),
            tool: Tool::flat_end_mill("endmill_3mm".to_string(), 10.0, 50.0),
            work_bounds: Aabb3D::new(
                Point3D::new(-60.0, -60.0, -20.0),
                Point3D::new(60.0, 60.0, 30.0),
            ),
            max_depth: 4,
            snapshot_interval: SnapshotInterval::default(),
        };

        let result =
            CamSimulationExecutionOrchestrator.execute_simulation_snapshot_exports(request);
        assert_eq!(
            result,
            Err(ApplicationError::Simulation(
                SimulationError::EmptyToolpath.to_string()
            ))
        );
    }

    #[test]
    fn test_application_error_displays_tool_entity_error() {
        let err = ApplicationError::ToolEntity("invalid tool".to_string());
        assert_eq!(err.to_string(), "invalid tool");
    }

    #[test]
    fn test_tool_entity_management_add_tool_to_storage() {
        let tool = Tool::flat_end_mill("test_tool".to_string(), 10.0, 50.0);
        let request = AddToolToEntityStorageRequest {
            tool,
            feature_id: "tool_import".to_string(),
            output_index: 0,
            local_key: "primary".to_string(),
            description: Some("Test Tool".to_string()),
        };

        let result = ToolEntityManagementOrchestrator
            .add_tool_to_storage(request)
            .expect("tool addition should succeed");

        assert_eq!(result.tool_name, "test_tool");
        assert_eq!(result.description, Some("Test Tool".to_string()));
        assert!(!result.entity_id.is_empty());
    }

    #[test]
    fn test_tool_entity_management_add_tool_to_storage_is_deterministic_for_same_feature_key() {
        let make_request = || AddToolToEntityStorageRequest {
            tool: Tool::flat_end_mill("test_tool".to_string(), 10.0, 50.0),
            feature_id: "tool_import".to_string(),
            output_index: 1,
            local_key: "primary".to_string(),
            description: Some("Test Tool".to_string()),
        };

        let first = ToolEntityManagementOrchestrator
            .add_tool_to_storage(make_request())
            .expect("first tool addition should succeed");
        let second = ToolEntityManagementOrchestrator
            .add_tool_to_storage(make_request())
            .expect("second tool addition should succeed");

        assert_eq!(first.entity_id, second.entity_id);
    }

    #[test]
    fn test_application_source_does_not_reference_demo_symbols() {
        demo_symbol_guard::assert_layer_does_not_reference_demo_symbols("application");
    }
}
