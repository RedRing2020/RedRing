use crate::error::SimulationError;

use super::{SnapshotConfig, SnapshotInterval};

impl<T: geo_algorithms::Scalar> super::CuttingSimulator<T> {
    /// スナップショット間隔設定を実行可能な値へ正規化する。
    ///
    /// - 不正な設定は `SimulationError::InvalidInterval` を返す
    /// - Auto指定は総距離から実間隔を算出する
    pub(super) fn calculate_snapshot_config(
        &self,
        total_distance: f64,
    ) -> Result<SnapshotConfig, SimulationError> {
        match self.interval {
            SnapshotInterval::ByDistance {
                interval_mm,
                include_segment_endpoints,
            } => {
                if interval_mm <= 0.0 {
                    return Err(SimulationError::InvalidInterval);
                }

                Ok(SnapshotConfig {
                    actual_interval_mm: interval_mm,
                    include_endpoints: include_segment_endpoints,
                })
            }
            SnapshotInterval::ByAutoDistance {
                target_count,
                min_interval_mm,
                include_segment_endpoints,
            } => {
                if target_count == 0 || min_interval_mm <= 0.0 {
                    return Err(SimulationError::InvalidInterval);
                }

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
