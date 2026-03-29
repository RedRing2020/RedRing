//! CAM-specific orchestration boundaries.

use cam_core::{Tool, ToolPath};
use cam_sim::{CuttingSimulator, SimulationError, SimulationSnapshotExport, SnapshotInterval};
use geo_algorithms::{Aabb3D, octree::VoxelOctree};

/// CAMシミュレーション由来の最小スナップショットDTO。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamSnapshotFrame {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// Request boundary for CAM snapshot series generation.
#[derive(Debug, Clone, PartialEq)]
pub struct CamSnapshotSeriesRequest {
    pub source: String,
    pub frames: Vec<CamSnapshotFrame>,
}

/// Result boundary for CAM snapshot series generation.
#[derive(Debug, Clone, PartialEq)]
pub struct CamSnapshotSeriesResult {
    pub frame_count: usize,
    pub source: String,
    pub frames: Vec<CamSnapshotFrame>,
}

/// Orchestration port for CAM snapshot-related use cases.
pub trait CamSnapshotSeriesOrchestration {
    fn create_snapshot_series(
        &self,
        request: CamSnapshotSeriesRequest,
    ) -> Result<CamSnapshotSeriesResult, String>;
}

/// Default implementation for snapshot series orchestration.
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

/// Request boundary for cam_sim snapshot execution.
#[derive(Debug, Clone)]
pub struct CamSimulationExecutionRequest {
    pub toolpath: ToolPath<f64>,
    pub tool: Tool<f64>,
    pub work_bounds: Aabb3D<f64>,
    pub max_depth: usize,
    pub snapshot_interval: SnapshotInterval,
}

/// Result boundary for cam_sim snapshot execution.
#[derive(Debug, Clone, PartialEq)]
pub struct CamSimulationExecutionResult {
    pub exports: Vec<SimulationSnapshotExport>,
}

/// Execute cam_sim and return snapshot exports as an application boundary result.
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

#[cfg(test)]
mod tests {
    use super::*;
    use cam_core::CuttingDirection;
    use geo_algorithms::Point3D;

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
}
