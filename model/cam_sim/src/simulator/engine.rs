use super::behavior::ToolCuttingBehavior;
use super::{PathPosition, SimulationSnapshot, SnapshotConfig};

impl<T: geo_algorithms::Scalar> super::CuttingSimulator<T> {
    /// 線分列を走査し、材料除去とスナップショット記録を実行する。
    pub(super) fn simulate_segments_with_flags(
        &mut self,
        segments: &[(geo_algorithms::LineSegment3D<T>, bool)],
        behavior: &dyn ToolCuttingBehavior<T>,
        config: SnapshotConfig,
    ) {
        self.snapshots.clear();
        if segments.is_empty() {
            return;
        }

        let segment_lengths: Vec<f64> = segments
            .iter()
            .map(|(segment, _)| self.segment_length(segment))
            .collect();

        let mut accumulated_distance = 0.0;
        let mut next_snapshot_distance = 0.0;

        for (index, (segment, is_cutting)) in segments.iter().enumerate() {
            let seg_length = segment_lengths[index];
            let seg_start_distance = accumulated_distance;

            if config.include_endpoints {
                self.save_snapshot(PathPosition {
                    segment_index: index,
                    t: 0.0,
                    accumulated_distance,
                });
            } else {
                while next_snapshot_distance <= accumulated_distance {
                    self.save_snapshot(PathPosition {
                        segment_index: index,
                        t: 0.0,
                        accumulated_distance: next_snapshot_distance,
                    });
                    next_snapshot_distance += config.actual_interval_mm;
                    if config.actual_interval_mm <= f64::EPSILON {
                        break;
                    }
                }
            }

            if *is_cutting {
                behavior.remove_material(&mut self.voxel_tree, segment);
            }
            accumulated_distance += seg_length;

            if config.actual_interval_mm > f64::EPSILON {
                while next_snapshot_distance < accumulated_distance {
                    let dist_in_segment = next_snapshot_distance - seg_start_distance;
                    let t = if seg_length <= f64::EPSILON {
                        0.0
                    } else {
                        (dist_in_segment / seg_length).clamp(0.0, 1.0)
                    };

                    self.save_snapshot(PathPosition {
                        segment_index: index,
                        t,
                        accumulated_distance: next_snapshot_distance,
                    });
                    next_snapshot_distance += config.actual_interval_mm;
                }
            }

            if config.include_endpoints {
                self.save_snapshot(PathPosition {
                    segment_index: index,
                    t: 1.0,
                    accumulated_distance,
                });
            }
        }
    }

    /// 重複記録を抑制しながらスナップショットを追加する。
    pub(super) fn save_snapshot(&mut self, position: PathPosition) {
        if let Some(last) = self.snapshots.last()
            && last.position.segment_index == position.segment_index
            && (last.position.t - position.t).abs() <= 1e-12
        {
            return;
        }

        self.snapshots.push(SimulationSnapshot {
            position,
            remaining_volume: self.voxel_tree.remaining_volume(),
        });
    }
}
