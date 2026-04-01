//! snapshot_converter - 時系列スナップショットのドメインモデル変換
//!
//! ViewModel層で利用するための汎用スナップショットDTOを定義します。
//! ドメイン固有情報は `payload` に閉じ込め、CAM以外（例: プレス）にも
//! 同じ構造で適用できることを目的とします。

use application::cam_orchestration::{
    create_snapshot_series_from_exports, ApplicationError, CamSimulationExecutionOrchestration,
    CamSimulationExecutionOrchestrator, CamSimulationExecutionRequest,
};
use cam_core::{fixtures::create_sample_toolpath, Tool};
use cam_sim::{SimulationSnapshotExport, SnapshotInterval};
use geo_algorithms::{Aabb3D, Point3D};

/// 3D姿勢情報（位置 + 任意の姿勢）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnapshotPose {
    /// 位置（mm）
    pub position_mm: [f64; 3],
    /// 姿勢クォータニオン（x, y, z, w）。未使用時は `None`
    pub orientation_xyzw: Option<[f64; 4]>,
}

/// スナップショット指標（例: 体積、荷重、ストローク）
#[derive(Debug, Clone, PartialEq)]
pub struct SnapshotMetric {
    pub key: String,
    pub value: f64,
    pub unit: Option<String>,
}

impl SnapshotMetric {
    pub fn new(key: impl Into<String>, value: f64, unit: Option<impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            value,
            unit: unit.map(|u| u.into()),
        }
    }
}

/// イベント重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotEventLevel {
    Info,
    Warning,
    Error,
}

/// スナップショットに紐づくイベント
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEvent {
    pub code: String,
    pub message: String,
    pub level: SnapshotEventLevel,
}

/// 1フレーム分のドメインスナップショット
#[derive(Debug, Clone, PartialEq)]
pub struct DomainSnapshotFrame<TPayload> {
    pub frame_index: usize,
    pub time_sec: Option<f64>,
    pub pose: Option<SnapshotPose>,
    pub metrics: Vec<SnapshotMetric>,
    pub events: Vec<SnapshotEvent>,
    pub payload: TPayload,
}

/// 時系列スナップショット列
#[derive(Debug, Clone, PartialEq)]
pub struct DomainSnapshotSeries<TPayload> {
    pub source: String,
    pub duration_sec: Option<f64>,
    pub frames: Vec<DomainSnapshotFrame<TPayload>>,
}

impl<TPayload> DomainSnapshotSeries<TPayload> {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            duration_sec: None,
            frames: Vec::new(),
        }
    }
}

/// CAM切削シミュレーション由来の最小スナップショット入力
///
/// cam_sim への直接依存を持たず、境界越しに必要情報のみ受け取るためのDTO。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamSimulationSnapshotInput {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// CAM入力列をドメインスナップショットへ変換
pub fn cam_snapshot_inputs_to_domain_series(
    source: impl Into<String>,
    inputs: &[CamSimulationSnapshotInput],
) -> DomainSnapshotSeries<CamSimulationSnapshotInput> {
    let mut series = DomainSnapshotSeries::new(source);

    series.frames = inputs
        .iter()
        .enumerate()
        .map(|(frame_index, input)| DomainSnapshotFrame {
            frame_index,
            time_sec: None,
            pose: None,
            metrics: vec![
                SnapshotMetric::new(
                    "accumulated_distance_mm",
                    input.accumulated_distance_mm,
                    Some("mm"),
                ),
                SnapshotMetric::new(
                    "remaining_volume_mm3",
                    input.remaining_volume_mm3,
                    Some("mm3"),
                ),
            ],
            events: Vec::new(),
            payload: *input,
        })
        .collect();

    series
}

/// cam_sim 側のエクスポートDTO相当データ（segment_index, t, distance, volume）から
/// `CamSimulationSnapshotInput` 列へ変換する補助関数。
pub fn cam_export_rows_to_inputs(
    rows: &[(usize, f64, f64, f64)],
) -> Vec<CamSimulationSnapshotInput> {
    rows.iter()
        .map(
            |(segment_index, segment_t, accumulated_distance_mm, remaining_volume_mm3)| {
                CamSimulationSnapshotInput {
                    segment_index: *segment_index,
                    segment_t: *segment_t,
                    accumulated_distance_mm: *accumulated_distance_mm,
                    remaining_volume_mm3: *remaining_volume_mm3,
                }
            },
        )
        .collect()
}

/// cam_sim のエクスポートDTOから `CamSimulationSnapshotInput` 列へ変換する。
pub fn cam_snapshot_exports_to_inputs(
    exports: &[SimulationSnapshotExport],
) -> Vec<CamSimulationSnapshotInput> {
    let result = create_snapshot_series_from_exports("cam_sim", exports)
        .expect("snapshot series conversion in application layer should not fail");

    result
        .frames
        .into_iter()
        .map(|frame| CamSimulationSnapshotInput {
            segment_index: frame.segment_index,
            segment_t: frame.segment_t,
            accumulated_distance_mm: frame.accumulated_distance_mm,
            remaining_volume_mm3: frame.remaining_volume_mm3,
        })
        .collect()
}

/// デバッグ用：cam_sim 実行結果からドメインスナップショット系列を生成する。
pub fn create_sample_cam_snapshot_domain_series(
) -> Result<DomainSnapshotSeries<CamSimulationSnapshotInput>, ApplicationError> {
    let bounds = Aabb3D::new(
        Point3D::new(-60.0, -60.0, -20.0),
        Point3D::new(60.0, 60.0, 30.0),
    );
    let toolpath = create_sample_toolpath();
    let tool = Tool::flat_end_mill("endmill_3mm".to_string(), 10.0, 50.0);

    let exports = CamSimulationExecutionOrchestrator
        .execute_simulation_snapshot_exports(CamSimulationExecutionRequest {
            toolpath,
            tool,
            work_bounds: bounds,
            max_depth: 4,
            snapshot_interval: SnapshotInterval::default(),
        })?
        .exports;
    let inputs = cam_snapshot_exports_to_inputs(&exports);

    Ok(cam_snapshot_inputs_to_domain_series("cam_sim", &inputs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cam_snapshot_inputs_to_domain_series_preserves_order() {
        let inputs = vec![
            CamSimulationSnapshotInput {
                segment_index: 0,
                segment_t: 0.0,
                accumulated_distance_mm: 0.0,
                remaining_volume_mm3: 1000.0,
            },
            CamSimulationSnapshotInput {
                segment_index: 0,
                segment_t: 0.5,
                accumulated_distance_mm: 10.0,
                remaining_volume_mm3: 980.0,
            },
        ];

        let series = cam_snapshot_inputs_to_domain_series("cam_sim", &inputs);

        assert_eq!(series.source, "cam_sim");
        assert_eq!(series.frames.len(), 2);
        assert_eq!(series.frames[0].frame_index, 0);
        assert_eq!(series.frames[1].frame_index, 1);
        assert_eq!(series.frames[1].payload.accumulated_distance_mm, 10.0);
    }

    #[test]
    fn test_cam_snapshot_inputs_to_domain_series_has_default_metrics() {
        let inputs = vec![CamSimulationSnapshotInput {
            segment_index: 3,
            segment_t: 1.0,
            accumulated_distance_mm: 42.0,
            remaining_volume_mm3: 750.0,
        }];

        let series = cam_snapshot_inputs_to_domain_series("cam_sim", &inputs);
        let metrics = &series.frames[0].metrics;

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].key, "accumulated_distance_mm");
        assert_eq!(metrics[1].key, "remaining_volume_mm3");
    }

    #[test]
    fn test_cam_export_rows_to_inputs() {
        let rows = vec![(1usize, 0.25f64, 12.0f64, 900.0f64)];
        let inputs = cam_export_rows_to_inputs(&rows);

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].segment_index, 1);
        assert_eq!(inputs[0].segment_t, 0.25);
        assert_eq!(inputs[0].accumulated_distance_mm, 12.0);
        assert_eq!(inputs[0].remaining_volume_mm3, 900.0);
    }

    #[test]
    fn test_cam_snapshot_exports_to_inputs() {
        let exports = vec![SimulationSnapshotExport {
            segment_index: 2,
            segment_t: 0.75,
            accumulated_distance_mm: 33.0,
            remaining_volume_mm3: 812.0,
        }];

        let inputs = cam_snapshot_exports_to_inputs(&exports);
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].segment_index, 2);
        assert_eq!(inputs[0].segment_t, 0.75);
    }

    #[test]
    fn test_create_sample_cam_snapshot_domain_series() {
        let series = create_sample_cam_snapshot_domain_series().expect("cam_sim sample should run");
        assert_eq!(series.source, "cam_sim");
        assert!(!series.frames.is_empty());
    }
}
