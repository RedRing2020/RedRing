use crate::error::SimulationError;

use super::{SnapshotConfig, SnapshotInterval};

impl super::CuttingSimulator {
    /// スナップショット間隔設定を実行可能な値へ正規化する。
    ///
    /// - 不正な設定は `SimulationError::InvalidInterval` を返す
    /// - Auto指定は総距離から実間隔を算出する
    /// - 工具と格子からの指定は `tool_and_grid_interval_mm` を間隔とする
    pub(super) fn calculate_snapshot_config(
        &self,
        total_distance: f64,
        tool_and_grid_interval_mm: f64,
    ) -> Result<SnapshotConfig, SimulationError> {
        if !self.interval.is_valid() {
            return Err(SimulationError::InvalidInterval);
        }
        match self.interval {
            SnapshotInterval::ByToolAndGrid => Ok(SnapshotConfig {
                actual_interval_mm: tool_and_grid_interval_mm,
                include_endpoints: false,
            }),
            SnapshotInterval::ByDistance {
                interval_mm,
                include_segment_endpoints,
            } => Ok(SnapshotConfig {
                actual_interval_mm: interval_mm,
                include_endpoints: include_segment_endpoints,
            }),
            SnapshotInterval::ByAutoDistance {
                target_count,
                min_interval_mm,
                include_segment_endpoints,
            } => {
                let target = target_count.max(1) as f64;
                let ideal = if total_distance <= f64::EPSILON {
                    min_interval_mm
                } else {
                    total_distance / target
                };

                Ok(SnapshotConfig {
                    actual_interval_mm: ideal.max(min_interval_mm).max(f64::EPSILON),
                    include_endpoints: include_segment_endpoints,
                })
            }
        }
    }
}
