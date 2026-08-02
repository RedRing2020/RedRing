use std::collections::HashMap;
use std::time::Instant;

use cam_core::{Tool, ToolPath};
use geo_algorithms::octree::VoxelOctree;
use geo_algorithms::{
    Aabb3D, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, LineSegment3D, Point3D, Scalar,
    default_kernel_numerical_zero_tolerance,
};

use crate::error::SimulationError;
use crate::exact_work::{
    ExactToolPrimitive, ExactWorkModel, PrimitiveSetExactWork, exact_work_conservative_margin,
};

mod behavior;
mod config;
mod engine;
mod segments;

use behavior::{FlatEndMillBehavior, tool_cutting_behavior};

pub use segments::{
    collect_toolpath_line_segments, collect_toolpath_line_segments_with_arc_options,
};

/// スナップショット保存間隔の指定方法。
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

/// 経路上の位置（どのセグメントのどこか）を表す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPosition {
    pub segment_index: usize,
    pub t: f64,
    pub accumulated_distance: f64,
}

/// 1時点のシミュレーション結果。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshot<T: Scalar> {
    pub position: PathPosition,
    pub remaining_volume: T,
}

/// ViewModel連携用のスナップショット出力DTO（単位付きのf64へ正規化）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimulationSnapshotExport {
    pub segment_index: usize,
    pub segment_t: f64,
    pub accumulated_distance_mm: f64,
    pub remaining_volume_mm3: f64,
}

/// 切削シミュレーション本体。
///
/// - 入力: ToolPath + Tool
/// - 出力: スナップショット列と更新済みVoxelOctree
#[derive(Debug, Clone)]
pub struct CuttingSimulator<T: Scalar> {
    voxel_tree: VoxelOctree<T>,
    interval: SnapshotInterval,
    snapshots: Vec<SimulationSnapshot<T>>,
    /// Arc セグメントを polyline 近似する際の弦誤差上限（mm）。
    arc_chord_tolerance_mm: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SnapshotConfig {
    pub(super) actual_interval_mm: f64,
    pub(super) include_endpoints: bool,
}

impl<T: Scalar> CuttingSimulator<T> {
    /// 新しいシミュレータを作成する。
    pub fn new(voxel_tree: VoxelOctree<T>, interval: SnapshotInterval) -> Self {
        Self {
            voxel_tree,
            interval,
            snapshots: Vec::new(),
            arc_chord_tolerance_mm: DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
        }
    }

    /// 内部VoxelOctreeへの参照を取得する。
    pub fn voxel_tree(&self) -> &VoxelOctree<T> {
        &self.voxel_tree
    }

    /// 内部VoxelOctreeへの可変参照を取得する。
    pub fn voxel_tree_mut(&mut self) -> &mut VoxelOctree<T> {
        &mut self.voxel_tree
    }

    /// 記録済みスナップショットを取得する。
    pub fn snapshots(&self) -> &[SimulationSnapshot<T>] {
        &self.snapshots
    }

    /// 記録済みスナップショットをViewModel連携向けDTOへ変換して返す。
    pub fn snapshot_exports_f64(&self) -> Vec<SimulationSnapshotExport> {
        self.snapshots
            .iter()
            .map(|snapshot| SimulationSnapshotExport {
                segment_index: snapshot.position.segment_index,
                segment_t: snapshot.position.t,
                accumulated_distance_mm: snapshot.position.accumulated_distance,
                remaining_volume_mm3: snapshot.remaining_volume.to_f64(),
            })
            .collect()
    }

    /// スナップショットをクリアする。
    pub fn clear_snapshots(&mut self) {
        self.snapshots.clear();
    }

    /// ToolPathをシミュレーションする。
    pub fn simulate(
        &mut self,
        toolpath: &ToolPath<T>,
        tool: &Tool<T>,
    ) -> Result<(), SimulationError> {
        let behavior = tool_cutting_behavior(tool)?;

        let segments = self.collect_line_segments(toolpath)?;
        if segments.is_empty() {
            return Err(SimulationError::EmptyToolpath);
        }

        let simulation_segments = behavior.prepare_segments(segments);

        let config = self.calculate_snapshot_config(
            simulation_segments
                .iter()
                .map(|(segment, _)| self.segment_length(segment))
                .sum(),
        )?;

        self.simulate_segments_with_flags(&simulation_segments, behavior.as_ref(), config);
        Ok(())
    }

    /// 線分列を直接シミュレーションする補助API。
    pub fn simulate_line_segments(&mut self, segments: &[LineSegment3D<T>], tool_radius: T) {
        let config = self
            .calculate_snapshot_config(
                segments
                    .iter()
                    .map(|segment| self.segment_length(segment))
                    .sum(),
            )
            .unwrap_or(SnapshotConfig {
                actual_interval_mm: 10.0,
                include_endpoints: true,
            });

        let flagged: Vec<(LineSegment3D<T>, bool)> = segments
            .iter()
            .cloned()
            .map(|segment| (segment, true))
            .collect();
        let behavior = FlatEndMillBehavior {
            radius: tool_radius,
        };
        self.simulate_segments_with_flags(&flagged, &behavior, config);
    }
}

/// 候補絞り（Voxel）と確定判定（ExactWork）の比較メトリクス。
#[derive(Debug, Clone, Copy)]
pub struct HybridGateMetrics {
    pub removed_voxel: f64,
    pub removed_exact: f64,
    pub gap: f64,
    pub boundary_disagreement_rate: f64,
    pub boundary_sample_count: usize,
    pub boundary_exact_only_count: usize,
    pub boundary_voxel_only_count: usize,
    pub elapsed_voxel_ms: f64,
    pub elapsed_exact_ms: f64,
    pub elapsed_ratio: f64,
}

/// ハイブリッド判定の評価設定。
#[derive(Debug, Clone, Copy)]
pub struct HybridGateConfig {
    pub bounds: Aabb3D<f64>,
    pub octree_depth: usize,
    pub sample_pitch: f64,
}

impl Default for HybridGateConfig {
    fn default() -> Self {
        Self {
            bounds: Aabb3D::new(
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(100.0, 100.0, 100.0),
            ),
            octree_depth: 5,
            sample_pitch: 2.0,
        }
    }
}

/// 候補絞り（Voxel）+確定判定（ExactWork）の統合評価を実行する。
pub fn run_hybrid_gate_case(
    toolpath: &ToolPath<f64>,
    tool: &Tool<f64>,
    sample_pitch: f64,
) -> Result<HybridGateMetrics, SimulationError> {
    let config = HybridGateConfig {
        sample_pitch,
        ..HybridGateConfig::default()
    };
    run_hybrid_gate_case_with_config(toolpath, tool, config)
}

/// 設定付きで候補絞り（Voxel）+確定判定（ExactWork）の統合評価を実行する。
pub fn run_hybrid_gate_case_with_config(
    toolpath: &ToolPath<f64>,
    tool: &Tool<f64>,
    config: HybridGateConfig,
) -> Result<HybridGateMetrics, SimulationError> {
    validate_hybrid_config(&config)?;

    let bounds = config.bounds;
    let initial_volume = bounds.volume();

    let voxel_started = Instant::now();
    let mut simulator = CuttingSimulator::new(
        VoxelOctree::new(bounds, config.octree_depth),
        SnapshotInterval::default(),
    );
    simulator.simulate(toolpath, tool)?;
    let elapsed_voxel_ms = voxel_started.elapsed().as_secs_f64() * 1000.0;

    let voxel_pitch = simulator.voxel_tree().voxel_size_at_max_depth();
    let exact_sample_pitch = config.sample_pitch.max(voxel_pitch);
    let boundary_band = voxel_pitch;
    let removed_voxel = initial_volume - simulator.voxel_tree().remaining_volume();
    let solid_bounds = simulator.voxel_tree().collect_solid_voxel_bounds();
    let voxel_index = SolidBoundsSpatialIndex::from_bounds(solid_bounds, exact_sample_pitch);

    let exact_started = Instant::now();
    let exact_work = build_exact_work(bounds, toolpath, tool)?;
    let removed_exact = initial_volume - exact_work.estimate_remaining_volume(exact_sample_pitch);
    let elapsed_exact_ms = exact_started.elapsed().as_secs_f64() * 1000.0;

    let gap = if removed_voxel.abs() <= f64::EPSILON {
        0.0
    } else {
        ((removed_exact - removed_voxel).abs()) / removed_voxel.abs()
    };
    let exact_conservative_margin = if !tool.is_ball_end_mill() && exact_work.primitive_count() > 1
    {
        exact_sample_pitch * 0.5
    } else {
        exact_work_conservative_margin(exact_sample_pitch)
    };

    let (
        boundary_rate,
        boundary_sample_count,
        boundary_exact_only_count,
        boundary_voxel_only_count,
    ) = compute_boundary_disagreement_rate(
        &bounds,
        exact_sample_pitch,
        boundary_band,
        exact_conservative_margin,
        &exact_work,
        &voxel_index,
    );

    Ok(HybridGateMetrics {
        removed_voxel,
        removed_exact,
        gap,
        boundary_disagreement_rate: boundary_rate,
        boundary_sample_count,
        boundary_exact_only_count,
        boundary_voxel_only_count,
        elapsed_voxel_ms,
        elapsed_exact_ms,
        elapsed_ratio: elapsed_exact_ms / elapsed_voxel_ms.max(1.0e-9),
    })
}

fn validate_hybrid_config(config: &HybridGateConfig) -> Result<(), SimulationError> {
    if config.octree_depth >= i32::BITS as usize {
        return Err(SimulationError::InvalidOctreeDepth);
    }
    if !point_is_finite(&config.bounds.min()) || !point_is_finite(&config.bounds.max()) {
        return Err(SimulationError::InvalidHybridBounds);
    }
    if config.bounds.is_empty() {
        return Err(SimulationError::InvalidHybridBounds);
    }
    if !config.sample_pitch.is_finite() || config.sample_pitch <= 0.0 {
        return Err(SimulationError::InvalidSamplePitch);
    }
    Ok(())
}

fn point_is_finite(point: &Point3D<f64>) -> bool {
    point.x().is_finite() && point.y().is_finite() && point.z().is_finite()
}

fn build_exact_work(
    bounds: Aabb3D<f64>,
    toolpath: &ToolPath<f64>,
    tool: &Tool<f64>,
) -> Result<PrimitiveSetExactWork, SimulationError> {
    let mut exact_work = PrimitiveSetExactWork::new(bounds);
    let segments =
        collect_toolpath_line_segments(toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);
    for (segment, is_cutting) in segments {
        if !is_cutting {
            continue;
        }

        if tool.is_flat_end_mill() {
            exact_work.apply_primitive(ExactToolPrimitive::Flat {
                segment,
                radius: tool.radius(),
            });
            continue;
        }

        if tool.is_ball_end_mill() {
            let start = Point3D::new(
                segment.start().x(),
                segment.start().y(),
                segment.start().z() + tool.radius(),
            );
            let end = Point3D::new(
                segment.end().x(),
                segment.end().y(),
                segment.end().z() + tool.radius(),
            );
            let ball_segment =
                LineSegment3D::new(start, end).ok_or(SimulationError::UnsupportedGeometry)?;
            exact_work.apply_primitive(ExactToolPrimitive::Ball {
                segment: ball_segment,
                radius: tool.radius(),
            });
            continue;
        }

        return Err(SimulationError::UnsupportedToolType);
    }
    Ok(exact_work)
}

const HYBRID_MAX_AXIS_SAMPLES: usize = 128;

fn hybrid_axis_sample_count(span: f64, sample_pitch: f64) -> usize {
    if !span.is_finite() || span <= 0.0 || !sample_pitch.is_finite() || sample_pitch <= 0.0 {
        return 1;
    }
    ((span / sample_pitch).ceil() as usize).clamp(1, HYBRID_MAX_AXIS_SAMPLES)
}

#[derive(Debug, Clone)]
struct SolidBoundsSpatialIndex {
    bucket_size: f64,
    buckets: HashMap<(i32, i32, i32), Vec<usize>>,
    solid_bounds: Vec<Aabb3D<f64>>,
}

impl SolidBoundsSpatialIndex {
    fn from_bounds(solid_bounds: Vec<Aabb3D<f64>>, bucket_size: f64) -> Self {
        let size = if bucket_size.is_finite() && bucket_size > 0.0 {
            bucket_size
        } else {
            1.0
        };

        let mut buckets: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
        for (index, bounds) in solid_bounds.iter().enumerate() {
            let min = bounds.min();
            let max = bounds.max();
            let min_ix = floor_bucket(min.x(), size);
            let min_iy = floor_bucket(min.y(), size);
            let min_iz = floor_bucket(min.z(), size);
            let max_ix = floor_bucket(max.x(), size);
            let max_iy = floor_bucket(max.y(), size);
            let max_iz = floor_bucket(max.z(), size);

            for ix in min_ix..=max_ix {
                for iy in min_iy..=max_iy {
                    for iz in min_iz..=max_iz {
                        buckets.entry((ix, iy, iz)).or_default().push(index);
                    }
                }
            }
        }

        Self {
            bucket_size: size,
            buckets,
            solid_bounds,
        }
    }

    fn contains_material_at(&self, point: &Point3D<f64>) -> bool {
        let key = (
            floor_bucket(point.x(), self.bucket_size),
            floor_bucket(point.y(), self.bucket_size),
            floor_bucket(point.z(), self.bucket_size),
        );

        self.buckets.get(&key).is_some_and(|candidate_indexes| {
            candidate_indexes
                .iter()
                .any(|&idx| self.solid_bounds[idx].contains_point(point))
        })
    }
}

fn floor_bucket(value: f64, bucket_size: f64) -> i32 {
    (value / bucket_size).floor() as i32
}

fn is_voxel_boundary_point(
    bounds: &Aabb3D<f64>,
    voxel_index: &SolidBoundsSpatialIndex,
    point: &Point3D<f64>,
    center_state: bool,
    probe_offset: f64,
) -> bool {
    let offsets = [
        (probe_offset, 0.0, 0.0),
        (-probe_offset, 0.0, 0.0),
        (0.0, probe_offset, 0.0),
        (0.0, -probe_offset, 0.0),
        (0.0, 0.0, probe_offset),
        (0.0, 0.0, -probe_offset),
    ];

    offsets.into_iter().any(|(dx, dy, dz)| {
        let p = Point3D::new(point.x() + dx, point.y() + dy, point.z() + dz);
        bounds.contains_point(&p) && voxel_index.contains_material_at(&p) != center_state
    })
}

fn compute_boundary_disagreement_rate(
    bounds: &Aabb3D<f64>,
    sample_pitch: f64,
    boundary_band: f64,
    exact_conservative_margin: f64,
    exact_work: &PrimitiveSetExactWork,
    voxel_index: &SolidBoundsSpatialIndex,
) -> (f64, usize, usize, usize) {
    if sample_pitch <= 0.0 || !sample_pitch.is_finite() {
        return (0.0, 0, 0, 0);
    }

    let min = bounds.min();
    let max = bounds.max();
    let width = max.x() - min.x();
    let height = max.y() - min.y();
    let depth = max.z() - min.z();

    let x_samples = hybrid_axis_sample_count(width, sample_pitch);
    let y_samples = hybrid_axis_sample_count(height, sample_pitch);
    let z_samples = hybrid_axis_sample_count(depth, sample_pitch);

    let dx = width / (x_samples as f64);
    let dy = height / (y_samples as f64);
    let dz = depth / (z_samples as f64);

    let probe_offset = if boundary_band.is_finite() && boundary_band > 0.0 {
        let epsilon = default_kernel_numerical_zero_tolerance::<f64>().max(f64::EPSILON);
        boundary_band * 0.5 + epsilon
    } else {
        sample_pitch * 0.5
    };
    let mut boundary_points = 0usize;
    let mut disagreements = 0usize;
    let mut exact_only = 0usize;
    let mut voxel_only = 0usize;
    for ix in 0..x_samples {
        let x = min.x() + ((ix as f64) + 0.5) * dx;
        for iy in 0..y_samples {
            let y = min.y() + ((iy as f64) + 0.5) * dy;
            for iz in 0..z_samples {
                let z = min.z() + ((iz as f64) + 0.5) * dz;
                let point = Point3D::new(x, y, z);
                let dist = exact_work.nearest_removed_surface_distance(&point);
                let voxel_has_material = voxel_index.contains_material_at(&point);

                if dist > boundary_band
                    && !is_voxel_boundary_point(
                        bounds,
                        voxel_index,
                        &point,
                        voxel_has_material,
                        probe_offset,
                    )
                {
                    continue;
                }

                boundary_points += 1;
                let exact_has_material =
                    exact_work.contains_material_at(&point) && dist > exact_conservative_margin;
                if exact_has_material != voxel_has_material {
                    disagreements += 1;
                    if exact_has_material {
                        exact_only += 1;
                    } else {
                        voxel_only += 1;
                    }
                }
            }
        }
    }

    if boundary_points == 0 {
        return (0.0, 0, 0, 0);
    }

    (
        disagreements as f64 / boundary_points as f64,
        boundary_points,
        exact_only,
        voxel_only,
    )
}
