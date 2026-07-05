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
        let samples: Vec<_> = records.iter().filter(|c| c.name == case_name).collect();
        let sample_count = samples.len() as f64;

        let avg_remaining = samples.iter().map(|c| c.remaining_volume).sum::<f64>() / sample_count;
        let avg_removed_ratio = samples.iter().map(|c| c.removed_ratio).sum::<f64>() / sample_count;
        let avg_elapsed_micros =
            samples.iter().map(|c| c.elapsed_micros as f64).sum::<f64>() / sample_count;

        eprintln!(
            "BASELINE case={} samples={} avg_remaining_volume={} avg_removed_ratio={} avg_elapsed_us={}",
            case_name,
            samples.len(),
            avg_remaining,
            avg_removed_ratio,
            avg_elapsed_micros
        );
    }
}
