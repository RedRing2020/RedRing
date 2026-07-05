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

#[test]
fn test_phase1_baseline_cases_are_deterministic() {
    let first = [
        run_box_partial_case(),
        run_capsule_basic_case(),
        run_z_axis_basic_case(),
    ];
    let second = [
        run_box_partial_case(),
        run_capsule_basic_case(),
        run_z_axis_basic_case(),
    ];

    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.name, b.name);
        assert!((a.remaining_volume - b.remaining_volume).abs() <= 1.0e-9);
        assert!((a.removed_ratio - b.removed_ratio).abs() <= 1.0e-12);
        assert!(a.elapsed_micros > 0);
        assert!(b.elapsed_micros > 0);
    }
}

#[test]
#[ignore = "baseline計測用: -- --ignored --nocapture で実行"]
fn measure_phase1_baseline_cases() {
    let mut records = Vec::new();

    for _ in 0..5 {
        records.push(run_box_partial_case());
        records.push(run_capsule_basic_case());
        records.push(run_z_axis_basic_case());
    }

    for case_name in [
        "box_partial_depth4",
        "capsule_basic_depth4",
        "z_axis_basic_depth4",
    ] {
        let samples: Vec<_> = records
            .iter()
            .filter(|c| c.name == case_name)
            .copied()
            .collect();
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
