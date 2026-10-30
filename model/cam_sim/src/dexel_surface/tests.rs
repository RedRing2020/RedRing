//! `DexelSurface` のテスト

use std::collections::HashMap;

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};

use super::*;
use crate::stock_model::StockModel;
use crate::tri_dexel::TriDexelModel;
use crate::{SimulationSnapshotExport, collect_toolpath_line_segments};

/// 同じ平面とみなす交点の座標の幅
const PRECISION: f64 = 0.01;

fn stock() -> Aabb3D<f64> {
    Aabb3D::new(Point3D::new(0.0, 0.0, -5.0), Point3D::new(20.0, 10.0, 0.0))
}

fn toolpath(points: &[Point3D<f64>], kind: SegmentType<f64>) -> ToolPath<f64> {
    ToolPath::new(
        "BEM3".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            points
                .windows(2)
                .map(|pair| PathSegment::new_line(pair[0], pair[1], kind))
                .collect(),
        )],
        vec![],
    )
}

/// 間隔 0.25 のトリデクセルで、掃引体ごとにスナップショットを置いた成果物と正本
fn artifact(
    points: &[Point3D<f64>],
    kind: SegmentType<f64>,
) -> (CuttingSimulationArtifact, StockModel) {
    let path = toolpath(points, kind);
    let tool = Tool::ball_end_mill("BEM3".to_string(), 3.0, 8.0);
    let segments = collect_toolpath_line_segments(&path, 0.01);
    let model = StockModel::from_segments(stock(), &segments, &tool).unwrap();
    let applied: Vec<usize> = (0..=model.primitives().len()).collect();
    let dexels = TriDexelModel::from_stock_model(&model, 0.25, &applied);
    let snapshots = applied
        .iter()
        .map(|&count| SimulationSnapshotExport {
            segment_index: count,
            segment_t: 0.0,
            accumulated_distance_mm: 0.0,
            remaining_volume_mm3: dexels.remaining_volume_at(count),
        })
        .collect();
    let artifact = CuttingSimulationArtifact {
        snapshots,
        grids: dexels.layouts(),
        snapshot_changes: dexels.snapshot_changes().to_vec(),
    };
    (artifact, model)
}

fn cut() -> (CuttingSimulationArtifact, StockModel) {
    artifact(
        &[
            Point3D::new(-2.0, 2.0, -1.0),
            Point3D::new(8.0, 5.0, -3.0),
            Point3D::new(15.0, 5.0, -3.0),
            Point3D::new(15.0, 12.0, -2.0),
        ],
        SegmentType::Cutting { feed_rate: 100.0 },
    )
}

/// 全ブロックの面を、頂点の位置で結合した三角形の列
fn all_triangles(surface: &DexelSurface) -> Vec<[[f64; 3]; 3]> {
    let [bx, by, bz] = surface.block_counts();
    let mut triangles = Vec::new();
    for x in 0..bx {
        for y in 0..by {
            for z in 0..bz {
                let mesh = surface.mesh_block([x, y, z]);
                for triangle in mesh.indices.chunks(3) {
                    triangles.push([0, 1, 2].map(|i| mesh.positions[triangle[i] as usize]));
                }
            }
        }
    }
    triangles
}

fn key(point: [f64; 3]) -> [i64; 3] {
    point.map(|value| (value * 1e6).round() as i64)
}

#[test]
fn surface_is_closed_across_blocks() {
    let (artifact, _) = cut();
    let surface = DexelSurface::at_snapshot(&artifact, artifact.snapshots.len() - 1, PRECISION);
    assert!(surface.block_counts().iter().product::<usize>() > 1);
    let triangles = all_triangles(&surface);
    assert!(!triangles.is_empty());
    // 向きのある辺ごとに、逆向きの辺がちょうど 1 つある（閉じていて、向きがそろっている）
    let mut edges: HashMap<([i64; 3], [i64; 3]), i32> = HashMap::new();
    for triangle in &triangles {
        for i in 0..3 {
            let (a, b) = (key(triangle[i]), key(triangle[(i + 1) % 3]));
            if a == b {
                continue;
            }
            *edges.entry((a, b)).or_default() += 1;
        }
    }
    for (&(a, b), &count) in &edges {
        assert_eq!(count, 1, "edge {a:?} -> {b:?}");
        assert_eq!(
            edges.get(&(b, a)),
            Some(&1),
            "edge {a:?} -> {b:?} has no twin"
        );
    }
}

#[test]
fn vertices_lie_on_the_stock_model_boundary() {
    let (artifact, model) = cut();
    let surface = DexelSurface::at_snapshot(&artifact, artifact.snapshots.len() - 1, PRECISION);
    let [bx, by, bz] = surface.block_counts();
    for x in 0..bx {
        for y in 0..by {
            for z in 0..bz {
                for position in surface.mesh_block([x, y, z]).positions {
                    // いずれかの軸の向きに少しずらすと、材料の内外が変わる
                    let on_boundary = (0..3).any(|axis| {
                        let shifted = |delta: f64| {
                            let mut point = position;
                            point[axis] += delta;
                            model.contains_material(Point3D::new(point[0], point[1], point[2]))
                        };
                        shifted(1e-6) != shifted(-1e-6)
                    });
                    assert!(on_boundary, "{position:?}");
                }
            }
        }
    }
}

#[test]
fn uncut_stock_is_a_box_with_outward_normals() {
    let (artifact, _) = artifact(
        &[Point3D::new(0.0, 0.0, 5.0), Point3D::new(1.0, 0.0, 5.0)],
        SegmentType::Rapid,
    );
    let surface = DexelSurface::at_snapshot(&artifact, 0, PRECISION);
    let center = [10.0, 5.0, -2.5];
    for triangle in all_triangles(&surface) {
        // 頂点は素材の外周の面の上にある
        for point in triangle {
            let on_face = (point[0].abs() < 1e-9 || (point[0] - 20.0).abs() < 1e-9)
                || (point[1].abs() < 1e-9 || (point[1] - 10.0).abs() < 1e-9)
                || ((point[2] + 5.0).abs() < 1e-9 || point[2].abs() < 1e-9);
            assert!(on_face, "{point:?}");
        }
        let normal = cross(sub(triangle[1], triangle[0]), sub(triangle[2], triangle[0]));
        let centroid = [0, 1, 2].map(|i| (triangle[0][i] + triangle[1][i] + triangle[2][i]) / 3.0);
        assert!(dot(normal, sub(centroid, center)) > 0.0);
    }
}

#[test]
fn uncut_stock_has_two_triangles_per_block_face_and_sharp_corners() {
    let (artifact, _) = artifact(
        &[Point3D::new(0.0, 0.0, 5.0), Point3D::new(1.0, 0.0, 5.0)],
        SegmentType::Rapid,
    );
    let surface = DexelSurface::at_snapshot(&artifact, 0, PRECISION);
    let triangles = all_triangles(&surface);
    let [bx, by, bz] = surface.block_counts();
    assert_eq!(triangles.len(), 2 * 2 * (bx * by + bx * bz + by * bz));
    // 素材の角は面取りされない
    let corner = [20.0, 10.0, 0.0];
    assert!(triangles.iter().flatten().any(|&point| point == corner));
}

#[test]
fn uncut_blocks_give_the_same_mesh_as_meshing_cell_by_cell() {
    let (artifact, _) = cut();
    for snapshot in [0, artifact.snapshots.len() - 1] {
        let surface = DexelSurface::at_snapshot(&artifact, snapshot, PRECISION);
        let [bx, by, bz] = surface.block_counts();
        let blocks: Vec<[usize; 3]> = (0..bx)
            .flat_map(|x| (0..by).flat_map(move |y| (0..bz).map(move |z| [x, y, z])))
            .collect();
        // 最初の時点はすべて、最後の時点も削っていないブロックが残る
        let uncut = blocks
            .iter()
            .filter(|&&block| surface.is_uncut_block(block))
            .count();
        assert!(
            uncut > 0 && (snapshot == 0) == (uncut == blocks.len()),
            "{uncut} uncut blocks"
        );
        for block in blocks {
            assert_eq!(
                surface.mesh_block(block),
                surface.mesh_block_by_cells(block),
                "snapshot {snapshot} block {block:?}"
            );
        }
    }
}

#[test]
fn cuts_shallower_than_the_precision_are_merged_into_the_flat_face() {
    let triangles = |depth: f64, precision: f64| {
        let (artifact, _) = artifact(
            // Z の向きのデクセルの線（y = 5.125）の真上を通る
            &[
                Point3D::new(-2.0, 5.125, -depth),
                Point3D::new(22.0, 5.125, -depth),
            ],
            SegmentType::Cutting { feed_rate: 100.0 },
        );
        let surface = DexelSurface::at_snapshot(&artifact, artifact.snapshots.len() - 1, precision);
        all_triangles(&surface).len()
    };
    let uncut = triangles(-1.0, PRECISION);
    // 精度より浅い削り跡は平らな上面にまとめ、削っていない素材と同じ三角形の数になる
    assert_eq!(triangles(0.4 * PRECISION, PRECISION), uncut);
    // 精度を細かくすると、同じ削り跡が別の面になる
    assert!(triangles(0.4 * PRECISION, 1e-6 * PRECISION) > uncut);
}

#[test]
fn partial_rebuild_matches_a_full_rebuild_at_each_snapshot() {
    let (artifact, _) = cut();
    let mut surface = DexelSurface::at_snapshot(&artifact, 0, PRECISION);
    let [bx, by, bz] = surface.block_counts();
    let mut meshes: HashMap<[usize; 3], SurfaceMesh> = HashMap::new();
    for x in 0..bx {
        for y in 0..by {
            for z in 0..bz {
                meshes.insert([x, y, z], surface.mesh_block([x, y, z]));
            }
        }
    }
    for snapshot in 1..artifact.snapshots.len() {
        for block in surface.apply_changes([artifact.snapshot_changes[snapshot].as_slice()]) {
            meshes.insert(block, surface.mesh_block(block));
        }
        let expected = DexelSurface::at_snapshot(&artifact, snapshot, PRECISION);
        for (block, mesh) in &meshes {
            assert_eq!(
                mesh,
                &expected.mesh_block(*block),
                "snapshot {snapshot} block {block:?}"
            );
        }
    }
}

#[test]
fn searching_mixed_cells_gives_the_same_mesh_as_scanning_every_cell() {
    let (artifact, _) = cut();
    for snapshot in [
        0,
        artifact.snapshots.len() / 2,
        artifact.snapshots.len() - 1,
    ] {
        let surface = DexelSurface::at_snapshot(&artifact, snapshot, PRECISION);
        let [bx, by, bz] = surface.block_counts();
        for x in 0..bx {
            for y in 0..by {
                for z in 0..bz {
                    assert_eq!(
                        surface.mesh_block([x, y, z]),
                        surface.mesh_block_by_scanning([x, y, z]),
                        "snapshot {snapshot} block {:?}",
                        [x, y, z]
                    );
                }
            }
        }
    }
}

#[test]
fn reverse_changes_restore_the_previous_snapshot_with_a_partial_rebuild() {
    let (artifact, _) = cut();
    let reverse = artifact.reverse_snapshot_changes();
    assert_eq!(reverse.len(), artifact.snapshot_changes.len());
    assert!(reverse[0].is_empty());

    let last = artifact.snapshots.len() - 1;
    let mut surface = DexelSurface::at_snapshot(&artifact, last, PRECISION);
    let [bx, by, bz] = surface.block_counts();
    let mut meshes: HashMap<[usize; 3], SurfaceMesh> = HashMap::new();
    for x in 0..bx {
        for y in 0..by {
            for z in 0..bz {
                meshes.insert([x, y, z], surface.mesh_block([x, y, z]));
            }
        }
    }
    for snapshot in (1..=last).rev() {
        for block in surface.apply_changes([reverse[snapshot].as_slice()]) {
            meshes.insert(block, surface.mesh_block(block));
        }
        let expected = DexelSurface::at_snapshot(&artifact, snapshot - 1, PRECISION);
        assert_eq!(
            surface.intervals,
            expected.intervals,
            "snapshot {}",
            snapshot - 1
        );
        for (block, mesh) in &meshes {
            assert_eq!(
                mesh,
                &expected.mesh_block(*block),
                "snapshot {} block {block:?}",
                snapshot - 1
            );
        }
    }
}

#[test]
fn meshing_blocks_in_parallel_matches_meshing_them_one_by_one() {
    let (artifact, _) = cut();
    let surface = DexelSurface::at_snapshot(&artifact, artifact.snapshots.len() - 1, PRECISION);
    let [bx, by, bz] = surface.block_counts();
    let blocks: Vec<[usize; 3]> = (0..bx)
        .flat_map(|x| (0..by).flat_map(move |y| (0..bz).map(move |z| [x, y, z])))
        .collect();
    let parallel = surface.mesh_blocks(&blocks);
    assert_eq!(parallel.len(), blocks.len());
    for (block, mesh) in blocks.iter().zip(&parallel) {
        assert_eq!(mesh, &surface.mesh_block(*block), "block {block:?}");
    }
}

#[test]
fn unchanged_dexels_make_no_block_dirty() {
    let (artifact, _) = cut();
    let mut surface = DexelSurface::at_snapshot(&artifact, 1, PRECISION);
    // 同じ時点の変化をもう一度当てても、区間は変わらない
    assert!(
        surface
            .apply_changes([artifact.snapshot_changes[1].as_slice()])
            .is_empty()
    );
}
