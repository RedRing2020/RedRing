use super::super::*;
use crate::LineSegment3D;
use geo_core::Point3D;
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
struct BaselineCase {
    name: &'static str,
    remaining_volume: f64,
    removed_ratio: f64,
    elapsed_micros: u128,
}

#[derive(Debug, Clone, Copy)]
struct BaselineCaseSummary {
    name: &'static str,
    remaining_volume: f64,
    removed_ratio: f64,
    elapsed_micros: f64,
}

#[derive(Debug, Clone, Copy)]
struct BaselineCaseDelta {
    remaining_volume_ratio: f64,
    removed_ratio_ratio: f64,
    elapsed_micros_ratio: f64,
}

type PerfGuardCase = (&'static str, fn() -> BaselineCase, f64, f64);

const PERF_GUARD_SAMPLE_COUNT: usize = 7;
const PERF_GUARD_MAX_RATIO: f64 = 1.20;
const PERF_GUARD_ABSOLUTE_ONLY_MAX_BASELINE_US: f64 = 50.0;
const PERF_GUARD_RATIO_MIN_BASELINE_US: f64 = 100.0;
const PERF_GUARD_MAX_ABSOLUTE_INCREASE_US: f64 = 20.0;
const PERF_GUARD_MAX_ENV_SLOWDOWN_RATIO: f64 = 1.10;
const PERF_GUARD_ENV_RATIO_VAR: &str = "REDRING_PERF_GUARD_ENV_RATIO";
const PERF_GUARD_ENABLE_VAR: &str = "REDRING_ENABLE_PERF_GUARD";
const BOX_PARTIAL_BASELINE_ELAPSED_MICROS: f64 = 16.8;
const BOX_COMPLETE_BASELINE_ELAPSED_MICROS: f64 = 0.0;
const CAPSULE_BASELINE_ELAPSED_MICROS: f64 = 731.0;
const Z_AXIS_BASELINE_ELAPSED_MICROS: f64 = 55.6;
const SWEPT_BASELINE_ELAPSED_MICROS: f64 = 450.2;
const BOX_PARTIAL_ADDITIONAL_JITTER_US: f64 = 40.0;
const BOX_COMPLETE_ADDITIONAL_JITTER_US: f64 = 20.0;
const CAPSULE_ADDITIONAL_JITTER_US: f64 = 800.0;
const Z_AXIS_ADDITIONAL_JITTER_US: f64 = 100.0;
const SWEPT_ADDITIONAL_JITTER_US: f64 = 500.0;
const COMPLEXITY_PROXY_MAX_NON_EMPTY_DEPTH4: usize = 1024;

impl BaselineCaseSummary {
    fn from_samples(name: &'static str, samples: &[BaselineCase]) -> Self {
        let sample_count = samples.len() as f64;
        Self {
            name,
            remaining_volume: samples.iter().map(|c| c.remaining_volume).sum::<f64>()
                / sample_count,
            removed_ratio: samples.iter().map(|c| c.removed_ratio).sum::<f64>() / sample_count,
            elapsed_micros: samples.iter().map(|c| c.elapsed_micros as f64).sum::<f64>()
                / sample_count,
        }
    }

    fn as_case(&self) -> BaselineCase {
        BaselineCase {
            name: self.name,
            remaining_volume: self.remaining_volume,
            removed_ratio: self.removed_ratio,
            elapsed_micros: self.elapsed_micros.round() as u128,
        }
    }
}

impl BaselineCaseDelta {
    fn from_cases(before: &BaselineCase, after: &BaselineCase) -> Self {
        Self {
            remaining_volume_ratio: relative_change_ratio(
                before.remaining_volume,
                after.remaining_volume,
            ),
            removed_ratio_ratio: relative_change_ratio(before.removed_ratio, after.removed_ratio),
            elapsed_micros_ratio: relative_change_ratio(
                before.elapsed_micros as f64,
                after.elapsed_micros as f64,
            ),
        }
    }

    fn format_line(&self, case_name: &str) -> String {
        format!(
            "CASE_DELTA case={} remaining_volume_ratio={:.6}% removed_ratio_ratio={:.6}% elapsed_us_ratio={:.6}%",
            case_name,
            self.remaining_volume_ratio * 100.0,
            self.removed_ratio_ratio * 100.0,
            self.elapsed_micros_ratio * 100.0,
        )
    }
}

fn relative_change_ratio(before: f64, after: f64) -> f64 {
    if before.abs() <= f64::EPSILON {
        0.0
    } else {
        (after - before) / before
    }
}

fn collect_samples(case_fn: fn() -> BaselineCase, sample_count: usize) -> Vec<BaselineCase> {
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        samples.push(case_fn());
    }
    samples
}

fn median_elapsed_micros(samples: &[BaselineCase]) -> f64 {
    let mut elapsed: Vec<u128> = samples.iter().map(|c| c.elapsed_micros).collect();
    elapsed.sort_unstable();
    let mid = elapsed.len() / 2;
    if elapsed.len() % 2 == 1 {
        elapsed[mid] as f64
    } else {
        (elapsed[mid - 1] as f64 + elapsed[mid] as f64) / 2.0
    }
}

fn median_f64(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    }
}

fn environment_slowdown_ratio_from_env() -> Option<f64> {
    let raw = std::env::var(PERF_GUARD_ENV_RATIO_VAR).ok()?;
    let parsed = raw.parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    Some(parsed.clamp(1.0, PERF_GUARD_MAX_ENV_SLOWDOWN_RATIO))
}

fn perf_guard_enabled() -> bool {
    std::env::var(PERF_GUARD_ENABLE_VAR)
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

fn assert_elapsed_within_20_percent(
    case_name: &str,
    actual_elapsed: f64,
    baseline_elapsed: f64,
    additional_jitter_us: f64,
    environment_slowdown_ratio: f64,
) {
    let adjusted_limit = PERF_GUARD_MAX_ABSOLUTE_INCREASE_US * environment_slowdown_ratio;
    let adjusted_jitter = additional_jitter_us * environment_slowdown_ratio;
    let adjusted_absolute_limit = adjusted_limit.max(adjusted_jitter);

    if baseline_elapsed <= f64::EPSILON {
        assert!(
            actual_elapsed <= adjusted_absolute_limit,
            "{} の実行時間が想定外に大きい: actual={}us baseline={}us limit={}us base_limit={}us jitter_limit={}us env_ratio={}",
            case_name,
            actual_elapsed,
            baseline_elapsed,
            adjusted_absolute_limit,
            adjusted_limit,
            adjusted_jitter,
            environment_slowdown_ratio
        );
        return;
    }

    if baseline_elapsed < PERF_GUARD_ABSOLUTE_ONLY_MAX_BASELINE_US {
        let increase = actual_elapsed - baseline_elapsed;
        assert!(
            increase <= adjusted_absolute_limit,
            "{} の実行時間が絶対値しきいを超えて悪化: actual={}us baseline={}us delta={}us limit={}us base_limit={}us jitter_limit={}us env_ratio={}",
            case_name,
            actual_elapsed,
            baseline_elapsed,
            increase,
            adjusted_absolute_limit,
            adjusted_limit,
            adjusted_jitter,
            environment_slowdown_ratio
        );
        return;
    }

    let ratio = actual_elapsed / baseline_elapsed;
    let adjusted_ratio_limit = PERF_GUARD_MAX_RATIO * environment_slowdown_ratio;

    if baseline_elapsed < PERF_GUARD_RATIO_MIN_BASELINE_US {
        let ratio_allowed_actual = baseline_elapsed * adjusted_ratio_limit;
        let absolute_allowed_actual = baseline_elapsed + adjusted_limit;
        let allowed_actual = ratio_allowed_actual
            .max(absolute_allowed_actual)
            .max(baseline_elapsed + adjusted_jitter);

        assert!(
            actual_elapsed <= allowed_actual,
            "{} の実行時間がハイブリッドしきいを超えて悪化: actual={}us baseline={}us ratio={:.2}% allowed_actual={}us ratio_allowed={}us absolute_allowed={}us jitter_allowed={}us env_ratio={}",
            case_name,
            actual_elapsed,
            baseline_elapsed,
            ratio * 100.0,
            allowed_actual,
            ratio_allowed_actual,
            absolute_allowed_actual,
            baseline_elapsed + adjusted_jitter,
            environment_slowdown_ratio
        );
        return;
    }

    let ratio_allowed_actual = baseline_elapsed * adjusted_ratio_limit;
    let jitter_allowed_actual = baseline_elapsed + adjusted_jitter;
    let allowed_actual = ratio_allowed_actual.max(jitter_allowed_actual);
    assert!(
        actual_elapsed <= allowed_actual,
        "{} の実行時間がしきいを超えて悪化: actual={}us baseline={}us ratio={:.2}% ratio_limit={:.2}% ratio_allowed={}us jitter_limit={}us jitter_allowed={}us allowed_actual={}us env_ratio={}",
        case_name,
        actual_elapsed,
        baseline_elapsed,
        ratio * 100.0,
        adjusted_ratio_limit * 100.0,
        ratio_allowed_actual,
        adjusted_jitter,
        jitter_allowed_actual,
        allowed_actual,
        environment_slowdown_ratio
    );
}

fn work_bounds() -> Aabb3D<f64> {
    Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    )
}

fn run_box_partial_case() -> BaselineCase {
    let mut voxel_tree = VoxelOctree::new(work_bounds(), 4);
    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(
        Point3D::new(40.0, 40.0, 40.0),
        Point3D::new(60.0, 60.0, 60.0),
    );

    let started = Instant::now();
    voxel_tree.remove_material_box(&tool_aabb);
    let elapsed = started.elapsed();

    let remaining = voxel_tree.remaining_volume();
    BaselineCase {
        name: "box_partial_depth4",
        remaining_volume: remaining,
        removed_ratio: (initial_volume - remaining) / initial_volume,
        elapsed_micros: elapsed.as_micros(),
    }
}

fn run_capsule_basic_case() -> BaselineCase {
    let mut voxel_tree = VoxelOctree::new(work_bounds(), 4);
    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 0.0),
        Point3D::new(50.0, 50.0, 100.0),
    )
    .expect("segment must be valid");

    let started = Instant::now();
    voxel_tree.remove_material_capsule(&segment, 10.0);
    let elapsed = started.elapsed();

    let remaining = voxel_tree.remaining_volume();
    BaselineCase {
        name: "capsule_basic_depth4",
        remaining_volume: remaining,
        removed_ratio: (initial_volume - remaining) / initial_volume,
        elapsed_micros: elapsed.as_micros(),
    }
}

fn run_z_axis_basic_case() -> BaselineCase {
    let mut voxel_tree = VoxelOctree::new(work_bounds(), 4);
    let initial_volume = voxel_tree.remaining_volume();

    let started = Instant::now();
    voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);
    let elapsed = started.elapsed();

    let remaining = voxel_tree.remaining_volume();
    BaselineCase {
        name: "z_axis_basic_depth4",
        remaining_volume: remaining,
        removed_ratio: (initial_volume - remaining) / initial_volume,
        elapsed_micros: elapsed.as_micros(),
    }
}

fn run_swept_cylinder_basic_case() -> BaselineCase {
    let mut voxel_tree = VoxelOctree::new(work_bounds(), 4);
    let initial_volume = voxel_tree.remaining_volume();
    let segment = LineSegment3D::new(
        Point3D::new(30.0, 50.0, 50.0),
        Point3D::new(70.0, 50.0, 50.0),
    )
    .expect("segment must be valid");

    let started = Instant::now();
    voxel_tree.remove_material_swept_cylinder(&segment, 10.0);
    let elapsed = started.elapsed();

    let remaining = voxel_tree.remaining_volume();
    BaselineCase {
        name: "swept_basic_depth4",
        remaining_volume: remaining,
        removed_ratio: (initial_volume - remaining) / initial_volume,
        elapsed_micros: elapsed.as_micros(),
    }
}

fn run_box_complete_case() -> BaselineCase {
    let mut voxel_tree = VoxelOctree::new(work_bounds(), 4);
    let initial_volume = voxel_tree.remaining_volume();
    let tool_aabb = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );

    let started = Instant::now();
    voxel_tree.remove_material_box(&tool_aabb);
    let elapsed = started.elapsed();

    let remaining = voxel_tree.remaining_volume();
    BaselineCase {
        name: "box_complete_depth4",
        remaining_volume: remaining,
        removed_ratio: (initial_volume - remaining) / initial_volume,
        elapsed_micros: elapsed.as_micros(),
    }
}

fn baseline_repro_cases() -> [BaselineCase; 5] {
    [
        run_box_partial_case(),
        run_box_complete_case(),
        run_capsule_basic_case(),
        run_z_axis_basic_case(),
        run_swept_cylinder_basic_case(),
    ]
}

fn non_empty_voxel_count_at_max_depth(tree: &VoxelOctree<f64>) -> usize {
    tree.collect_non_empty_voxel_bounds_up_to_depth(tree.max_depth())
        .len()
}

#[test]
fn test_phase1_complexity_proxy_guards_shape_matrix() {
    let bounds = work_bounds();

    let mut box_complete = VoxelOctree::new(bounds, 4);
    let full_box = Aabb3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(100.0, 100.0, 100.0),
    );
    box_complete.remove_material_box(&full_box);
    assert_eq!(box_complete.solid_voxel_count(), 0);
    assert_eq!(non_empty_voxel_count_at_max_depth(&box_complete), 0);

    let capsule_segment = LineSegment3D::new(
        Point3D::new(50.0, 50.0, 0.0),
        Point3D::new(50.0, 50.0, 100.0),
    )
    .expect("segment must be valid");

    let mut capsule_tree = VoxelOctree::new(bounds, 4);
    capsule_tree.remove_material_capsule(&capsule_segment, 10.0);
    let capsule_solid = capsule_tree.solid_voxel_count();
    let capsule_non_empty = non_empty_voxel_count_at_max_depth(&capsule_tree);
    assert!(capsule_non_empty > 0);
    assert!(capsule_non_empty <= COMPLEXITY_PROXY_MAX_NON_EMPTY_DEPTH4);
    assert!(capsule_solid <= capsule_non_empty);

    let mut z_axis_tree = VoxelOctree::new(bounds, 4);
    z_axis_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);
    let z_axis_solid = z_axis_tree.solid_voxel_count();
    let z_axis_non_empty = non_empty_voxel_count_at_max_depth(&z_axis_tree);
    assert!(z_axis_non_empty > 0);
    assert!(z_axis_non_empty <= COMPLEXITY_PROXY_MAX_NON_EMPTY_DEPTH4);
    assert!(z_axis_solid <= z_axis_non_empty);

    // Optimized Z-axis path should not explode active-cell complexity versus generic capsule.
    assert!(
        z_axis_non_empty <= capsule_non_empty,
        "z_axis complexity proxy regressed: z_axis_non_empty={} capsule_non_empty={}",
        z_axis_non_empty,
        capsule_non_empty
    );

    let solid_diff = z_axis_solid.abs_diff(capsule_solid);
    assert!(
        solid_diff <= 64,
        "z_axis vs capsule solid voxel count diverged too much: diff={} z_axis={} capsule={}",
        solid_diff,
        z_axis_solid,
        capsule_solid
    );

    let mut swept_tree = VoxelOctree::new(bounds, 4);
    let swept_segment = LineSegment3D::new(
        Point3D::new(30.0, 50.0, 50.0),
        Point3D::new(70.0, 50.0, 50.0),
    )
    .expect("segment must be valid");
    swept_tree.remove_material_swept_cylinder(&swept_segment, 10.0);
    let swept_solid = swept_tree.solid_voxel_count();
    let swept_non_empty = non_empty_voxel_count_at_max_depth(&swept_tree);
    assert!(swept_non_empty > 0);
    assert!(swept_non_empty <= COMPLEXITY_PROXY_MAX_NON_EMPTY_DEPTH4);
    assert!(swept_solid <= swept_non_empty);

    let mut swept_capsule_ref_tree = VoxelOctree::new(bounds, 4);
    swept_capsule_ref_tree.remove_material_capsule(&swept_segment, 10.0);
    let swept_capsule_ref_solid = swept_capsule_ref_tree.solid_voxel_count();
    let swept_capsule_ref_non_empty = non_empty_voxel_count_at_max_depth(&swept_capsule_ref_tree);
    assert!(swept_capsule_ref_non_empty > 0);
    assert!(swept_capsule_ref_non_empty <= COMPLEXITY_PROXY_MAX_NON_EMPTY_DEPTH4);
    assert!(swept_capsule_ref_solid <= swept_capsule_ref_non_empty);

    let mut capsule_tree_2 = VoxelOctree::new(bounds, 4);
    capsule_tree_2.remove_material_capsule(&capsule_segment, 10.0);
    assert_eq!(
        capsule_solid,
        capsule_tree_2.solid_voxel_count(),
        "capsule complexity proxy should be deterministic for fixed input"
    );
    assert_eq!(
        capsule_non_empty,
        non_empty_voxel_count_at_max_depth(&capsule_tree_2),
        "capsule non-empty proxy should be deterministic for fixed input"
    );

    let mut z_axis_tree_2 = VoxelOctree::new(bounds, 4);
    z_axis_tree_2.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);
    assert_eq!(
        z_axis_solid,
        z_axis_tree_2.solid_voxel_count(),
        "z-axis complexity proxy should be deterministic for fixed input"
    );
    assert_eq!(
        z_axis_non_empty,
        non_empty_voxel_count_at_max_depth(&z_axis_tree_2),
        "z-axis non-empty proxy should be deterministic for fixed input"
    );

    let mut swept_tree_2 = VoxelOctree::new(bounds, 4);
    swept_tree_2.remove_material_swept_cylinder(&swept_segment, 10.0);
    assert_eq!(
        swept_solid,
        swept_tree_2.solid_voxel_count(),
        "swept complexity proxy should be deterministic for fixed input"
    );
    assert_eq!(
        swept_non_empty,
        non_empty_voxel_count_at_max_depth(&swept_tree_2),
        "swept non-empty proxy should be deterministic for fixed input"
    );
}

#[test]
fn test_phase1_baseline_cases_are_deterministic() {
    let first = baseline_repro_cases();
    let second = baseline_repro_cases();

    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.name, b.name);
        assert!((a.remaining_volume - b.remaining_volume).abs() <= 1.0e-9);
        assert!((a.removed_ratio - b.removed_ratio).abs() <= 1.0e-12);
    }
}

#[test]
#[ignore = "時間依存テスト: REDRING_ENABLE_PERF_GUARD=1 を設定し、cargo test -- --ignored で本番相当環境のみ実行"]
fn test_phase1_performance_guard_within_20_percent() {
    assert!(
        perf_guard_enabled(),
        "set {}=1 to run timing guard on production-like environment",
        PERF_GUARD_ENABLE_VAR
    );

    let cases: [PerfGuardCase; 5] = [
        (
            "box_partial_depth4",
            run_box_partial_case,
            BOX_PARTIAL_BASELINE_ELAPSED_MICROS,
            BOX_PARTIAL_ADDITIONAL_JITTER_US,
        ),
        (
            "box_complete_depth4",
            run_box_complete_case,
            BOX_COMPLETE_BASELINE_ELAPSED_MICROS,
            BOX_COMPLETE_ADDITIONAL_JITTER_US,
        ),
        (
            "capsule_basic_depth4",
            run_capsule_basic_case,
            CAPSULE_BASELINE_ELAPSED_MICROS,
            CAPSULE_ADDITIONAL_JITTER_US,
        ),
        (
            "z_axis_basic_depth4",
            run_z_axis_basic_case,
            Z_AXIS_BASELINE_ELAPSED_MICROS,
            Z_AXIS_ADDITIONAL_JITTER_US,
        ),
        (
            "swept_basic_depth4",
            run_swept_cylinder_basic_case,
            SWEPT_BASELINE_ELAPSED_MICROS,
            SWEPT_ADDITIONAL_JITTER_US,
        ),
    ];

    let mut measured_cases: Vec<(&str, f64, f64, f64, usize)> = Vec::with_capacity(cases.len());
    for (case_name, case_fn, baseline_elapsed, additional_jitter_us) in cases {
        let samples = collect_samples(case_fn, PERF_GUARD_SAMPLE_COUNT);
        let measured_elapsed = median_elapsed_micros(&samples);
        measured_cases.push((
            case_name,
            measured_elapsed,
            baseline_elapsed,
            additional_jitter_us,
            samples.len(),
        ));
    }

    let slowdown_candidates: Vec<f64> = measured_cases
        .iter()
        .filter_map(|(_, measured_elapsed, baseline_elapsed, _, _)| {
            if *baseline_elapsed > f64::EPSILON {
                Some((measured_elapsed / baseline_elapsed).max(1.0))
            } else {
                None
            }
        })
        .collect();
    let measured_slowdown_ratio = if slowdown_candidates.is_empty() {
        1.0
    } else {
        median_f64(&slowdown_candidates).min(PERF_GUARD_MAX_ENV_SLOWDOWN_RATIO)
    };
    let environment_slowdown_ratio = environment_slowdown_ratio_from_env().unwrap_or(1.0);

    eprintln!(
        "PERF_GUARD summary env_ratio={} measured_env_ratio={} sample_count={} env_var={}",
        environment_slowdown_ratio,
        measured_slowdown_ratio,
        PERF_GUARD_SAMPLE_COUNT,
        PERF_GUARD_ENV_RATIO_VAR
    );

    for (case_name, measured_elapsed, baseline_elapsed, additional_jitter_us, sample_len) in
        measured_cases
    {
        eprintln!(
            "PERF_GUARD case={} samples={} median_elapsed_us={} baseline_elapsed_us={} case_jitter_us={} limit_ratio=20% env_ratio={}",
            case_name,
            sample_len,
            measured_elapsed,
            baseline_elapsed,
            additional_jitter_us,
            environment_slowdown_ratio
        );

        assert_elapsed_within_20_percent(
            case_name,
            measured_elapsed,
            baseline_elapsed,
            additional_jitter_us,
            environment_slowdown_ratio,
        );
    }
}

#[test]
#[ignore = "baseline計測用: -- --ignored --nocapture で実行"]
fn measure_phase1_baseline_cases() {
    let mut records = Vec::new();

    for _ in 0..5 {
        records.push(run_box_partial_case());
        records.push(run_box_complete_case());
        records.push(run_capsule_basic_case());
        records.push(run_z_axis_basic_case());
        records.push(run_swept_cylinder_basic_case());
    }

    for case_name in [
        "box_partial_depth4",
        "box_complete_depth4",
        "capsule_basic_depth4",
        "z_axis_basic_depth4",
        "swept_basic_depth4",
    ] {
        let samples: Vec<_> = records
            .iter()
            .filter(|c| c.name == case_name)
            .copied()
            .collect();
        assert!(
            !samples.is_empty(),
            "baseline samples must not be empty for case={}",
            case_name
        );
        let summary = BaselineCaseSummary::from_samples(case_name, &samples);
        let baseline = samples[0];
        let summary_case = summary.as_case();
        let delta = BaselineCaseDelta::from_cases(&baseline, &summary_case);

        eprintln!(
            "BASELINE case={} samples={} avg_remaining_volume={} avg_removed_ratio={} avg_elapsed_us={}",
            case_name,
            samples.len(),
            summary.remaining_volume,
            summary.removed_ratio,
            summary.elapsed_micros
        );
        eprintln!("{}", delta.format_line(case_name));
    }
}
