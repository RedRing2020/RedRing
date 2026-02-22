use geo_algorithms::octree::VoxelOctree;
use geo_algorithms::{LineSegment3D, Scalar};

#[derive(Debug, Clone)]
pub enum SnapshotInterval {
    ByDistance {
        interval_mm: f64,
        include_segment_endpoints: bool,
    },
    ByAutoDistance {
        target_count: usize,
        min_interval_mm: f64,
        include_segment_endpoints: bool,
    },
}

impl Default for SnapshotInterval {
    fn default() -> Self {
        Self::ByDistance {
            interval_mm: 10.0,
            include_segment_endpoints: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPosition {
    pub segment_index: usize,
    pub t: f64,
    pub accumulated_distance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshot<T: Scalar> {
    pub position: PathPosition,
    pub remaining_volume: T,
}

#[derive(Debug, Clone)]
pub struct CuttingSimulator<T: Scalar> {
    voxel_tree: VoxelOctree<T>,
    interval: SnapshotInterval,
    snapshots: Vec<SimulationSnapshot<T>>,
}

#[derive(Debug, Clone, Copy)]
struct SnapshotConfig {
    actual_interval_mm: f64,
    include_endpoints: bool,
}

impl<T: Scalar> CuttingSimulator<T> {
    pub fn new(voxel_tree: VoxelOctree<T>, interval: SnapshotInterval) -> Self {
        Self {
            voxel_tree,
            interval,
            snapshots: Vec::new(),
        }
    }

    pub fn voxel_tree(&self) -> &VoxelOctree<T> {
        &self.voxel_tree
    }

    pub fn voxel_tree_mut(&mut self) -> &mut VoxelOctree<T> {
        &mut self.voxel_tree
    }

    pub fn snapshots(&self) -> &[SimulationSnapshot<T>] {
        &self.snapshots
    }

    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    pub fn simulate_line_segments(&mut self, segments: &[LineSegment3D<T>], tool_radius: T) {
        self.snapshots.clear();
        if segments.is_empty() {
            return;
        }

        let segment_lengths = self.segment_lengths(segments);
        let total_distance: f64 = segment_lengths.iter().sum();
        let config = self.calculate_snapshot_config(total_distance);

        let mut accumulated_distance = 0.0;
        let mut next_snapshot_distance = 0.0;

        for (index, segment) in segments.iter().enumerate() {
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

            self.voxel_tree
                .remove_material_capsule(segment, tool_radius);
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

    fn calculate_snapshot_config(&self, total_distance: f64) -> SnapshotConfig {
        match self.interval {
            SnapshotInterval::ByDistance {
                interval_mm,
                include_segment_endpoints,
            } => SnapshotConfig {
                actual_interval_mm: interval_mm.max(f64::EPSILON),
                include_endpoints: include_segment_endpoints,
            },
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

                SnapshotConfig {
                    actual_interval_mm: ideal.max(min_interval_mm).max(f64::EPSILON),
                    include_endpoints: include_segment_endpoints,
                }
            }
        }
    }

    fn save_snapshot(&mut self, position: PathPosition) {
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

    fn segment_lengths(&self, segments: &[LineSegment3D<T>]) -> Vec<f64> {
        segments
            .iter()
            .map(|segment| {
                let dx = segment.end().x() - segment.start().x();
                let dy = segment.end().y() - segment.start().y();
                let dz = segment.end().z() - segment.start().z();
                (dx * dx + dy * dy + dz * dz).sqrt().to_f64()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use geo_algorithms::{Aabb3D, Point3D};

    use super::*;

    #[test]
    fn test_simulate_line_segments_removes_material() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let voxel = VoxelOctree::new(bounds, 5);
        let initial = voxel.remaining_volume();

        let mut simulator = CuttingSimulator::new(voxel, SnapshotInterval::default());

        let segment = LineSegment3D::new(
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(90.0, 10.0, 10.0),
        )
        .unwrap();

        simulator.simulate_line_segments(&[segment], 5.0);

        let remaining = simulator.voxel_tree().remaining_volume();
        assert!(remaining < initial);
        assert!(!simulator.snapshots().is_empty());
    }

    #[test]
    fn test_auto_distance_interval_generates_snapshots() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let voxel = VoxelOctree::new(bounds, 4);

        let mut simulator = CuttingSimulator::new(
            voxel,
            SnapshotInterval::ByAutoDistance {
                target_count: 5,
                min_interval_mm: 1.0,
                include_segment_endpoints: false,
            },
        );

        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(50.0, 0.0, 0.0)).unwrap();

        simulator.simulate_line_segments(&[segment], 2.0);
        assert!(!simulator.snapshots().is_empty());
    }
}
