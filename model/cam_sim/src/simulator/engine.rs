use geo_algorithms::LineSegment3D;

use super::{PathPosition, SimulationSnapshot, SnapshotConfig, segment_length};

impl super::CuttingSimulator {
    /// 線分列を走査し、スナップショットの時点と、その時点までに差し引く切削の区間の数を記録する。
    ///
    /// 区間の始点の時点はその区間の切削の前、区間の途中と終点の時点はその区間の切削の後とする。
    pub(super) fn record_snapshots(
        &mut self,
        segments: &[(LineSegment3D<f64>, bool)],
        config: SnapshotConfig,
    ) {
        self.snapshots.clear();
        self.snapshot_applied.clear();

        let mut applied = 0;
        let mut accumulated_distance = 0.0;
        let mut next_snapshot_distance = 0.0;

        for (index, (segment, is_cutting)) in segments.iter().enumerate() {
            let seg_length = segment_length(segment);
            let seg_start_distance = accumulated_distance;

            if config.include_endpoints {
                self.save_snapshot(
                    PathPosition {
                        segment_index: index,
                        t: 0.0,
                        accumulated_distance,
                    },
                    applied,
                );
            } else {
                while next_snapshot_distance <= accumulated_distance {
                    self.save_snapshot(
                        PathPosition {
                            segment_index: index,
                            t: 0.0,
                            accumulated_distance: next_snapshot_distance,
                        },
                        applied,
                    );
                    next_snapshot_distance += config.actual_interval_mm;
                    if config.actual_interval_mm <= f64::EPSILON {
                        break;
                    }
                }
            }

            if *is_cutting {
                applied += 1;
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

                    self.save_snapshot(
                        PathPosition {
                            segment_index: index,
                            t,
                            accumulated_distance: next_snapshot_distance,
                        },
                        applied,
                    );
                    next_snapshot_distance += config.actual_interval_mm;
                }
            }

            if config.include_endpoints {
                self.save_snapshot(
                    PathPosition {
                        segment_index: index,
                        t: 1.0,
                        accumulated_distance,
                    },
                    applied,
                );
            }
        }
    }

    /// 重複記録を抑制しながらスナップショットを追加する（残りの体積は、トリデクセルを作った後に入れる）。
    fn save_snapshot(&mut self, position: PathPosition, applied: usize) {
        if let Some(last) = self.snapshots.last()
            && last.position.segment_index == position.segment_index
            && (last.position.t - position.t).abs() <= 1e-12
        {
            return;
        }

        self.snapshots.push(SimulationSnapshot {
            position,
            remaining_volume: 0.0,
        });
        self.snapshot_applied.push(applied);
    }
}
