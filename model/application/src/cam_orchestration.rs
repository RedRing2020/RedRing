//! CAM-specific orchestration boundaries.

use cam_sim::SimulationSnapshotExport;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
