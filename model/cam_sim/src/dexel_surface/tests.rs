//! `DexelSurface` のテスト

use std::collections::HashMap;

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};

use super::*;
use crate::simulation_artifact::CuttingSimulationArtifact;
use crate::stock_model::StockModel;
use crate::tri_dexel::TriDexelModel;
use crate::{SimulationSnapshotExport, collect_toolpath_line_segments};

/// 同じ平面とみなす交点の座標の幅
const PRECISION: f64 = 0.01;

/// 工具（ボールエンドミル φ3）の半径（削った面の曲率半径の最小）
const RADIUS: f64 = 1.5;

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

/// 間隔 0.25 のトリデクセルで、掃引体ごとにスナップショットを置いた成果物と正本（ボールエンドミル φ3）
fn artifact(
    points: &[Point3D<f64>],
    kind: SegmentType<f64>,
) -> (CuttingSimulationArtifact, StockModel) {
    artifact_with_tool(
        points,
        kind,
        &Tool::ball_end_mill("BEM3".to_string(), 2.0 * RADIUS, 8.0),
    )
}

/// 間隔 0.25 のトリデクセルで、工具 `tool` で削り、掃引体ごとにスナップショットを置いた成果物と正本
fn artifact_with_tool(
    points: &[Point3D<f64>],
    kind: SegmentType<f64>,
    tool: &Tool<f64>,
) -> (CuttingSimulationArtifact, StockModel) {
    let path = toolpath(points, kind);
    let segments = collect_toolpath_line_segments(&path, 0.01);
    let model = StockModel::from_segments(stock(), &segments, tool).unwrap();
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
    // 掃引体ごとの時点をすべてキーフレームにする
    let artifact = CuttingSimulationArtifact {
        snapshots,
        keyframe_snapshots: (0..applied.len()).collect(),
        snapshot_applied: applied,
        grids: dexels.layouts(),
        keyframe_changes: dexels.keyframe_changes().clone(),
    };
    (artifact, model)
}

/// すべての時点がキーフレームの成果物 `artifact` の、時点 `snapshot` の面
fn surface_at(
    artifact: &CuttingSimulationArtifact,
    snapshot: usize,
    flat_tolerance: f64,
    curvature_radius: f64,
) -> DexelSurface {
    DexelSurface::new(
        artifact.grids,
        artifact.dexel_states_at_keyframe(snapshot),
        flat_tolerance,
        curvature_radius,
    )
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
    let surface = surface_at(&artifact, artifact.snapshots.len() - 1, PRECISION, RADIUS);
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

/// 全ブロックの面の頂点の位置と陰影の法線
fn all_vertices(surface: &DexelSurface) -> Vec<([f64; 3], [f64; 3])> {
    let [bx, by, bz] = surface.block_counts();
    let mut vertices = Vec::new();
    for x in 0..bx {
        for y in 0..by {
            for z in 0..bz {
                let mesh = surface.mesh_block([x, y, z]);
                vertices.extend(mesh.positions.into_iter().zip(mesh.normals));
            }
        }
    }
    vertices
}

#[test]
fn vertices_lie_near_the_stock_model_boundary() {
    let (artifact, model) = cut();
    let surface = surface_at(&artifact, artifact.snapshots.len() - 1, PRECISION, RADIUS);
    // 頂点は接平面の交わりに置くため、曲面からは「間隔² / 曲率半径」程度まで離れうる
    let reach = 0.25 * 0.25 / RADIUS;
    // 座標軸・面の対角・体の対角の 13 方向（稜線の上の頂点は、軸の向きにずらすと両側とも面の上に乗る）
    let directions: Vec<[f64; 3]> = (0..27)
        .map(|code| [code % 3, code / 3 % 3, code / 9].map(|digit| f64::from(digit) - 1.0))
        .filter(|direction| {
            let first = direction.iter().find(|&&component| component != 0.0);
            first.is_some_and(|&component| component > 0.0)
        })
        .map(|direction| {
            let length = dot(direction, direction).sqrt();
            direction.map(|component| component / length)
        })
        .collect();
    for (position, _) in all_vertices(&surface) {
        // いずれかの向きに少しずらすと、材料の内外が変わる
        let near_boundary = directions.iter().any(|direction| {
            let shifted = |delta: f64| {
                let point: [f64; 3] =
                    std::array::from_fn(|axis| position[axis] + delta * direction[axis]);
                model.contains_material(Point3D::new(point[0], point[1], point[2]))
            };
            shifted(reach) != shifted(-reach)
        });
        assert!(near_boundary, "{position:?}");
    }
}

#[test]
fn flat_end_mill_floor_edges_are_kept_sharp() {
    // フラットエンドミル φ4 で深さ 2 の水平の溝（底 z = -2、壁 y = 3・7）を削る。格子線は稜線を通らない
    let (artifact, _) = artifact_with_tool(
        &[Point3D::new(-5.0, 5.0, -2.0), Point3D::new(25.0, 5.0, -2.0)],
        SegmentType::Cutting { feed_rate: 100.0 },
        &Tool::flat_end_mill("FEM4".to_string(), 4.0, 8.0),
    );
    let surface = surface_at(&artifact, artifact.snapshots.len() - 1, PRECISION, 2.0);
    let vertices = all_vertices(&surface);
    for wall in [3.0, 7.0] {
        let on_edge: Vec<[f64; 3]> = vertices
            .iter()
            .filter(|(position, _)| {
                (position[1] - wall).abs() < 1e-9 && (position[2] + 2.0).abs() < 1e-9
            })
            .map(|&(_, normal)| normal)
            .collect();
        // 稜線の上に頂点があり（面取りされない）、底と壁で陰影の頂点が分かれる
        assert!(
            on_edge.len() >= 4,
            "wall {wall}: {} vertices",
            on_edge.len()
        );
        assert!(on_edge.iter().any(|normal| normal[2] > 0.99), "wall {wall}");
        assert!(
            on_edge.iter().any(|normal| normal[1].abs() > 0.99),
            "wall {wall}"
        );
    }
}

#[test]
fn ball_end_mill_cusps_are_not_chamfered() {
    // ボールエンドミル φ3 で、ステップオーバー 1 の 2 本の溝を削る。カスプの稜線 y = 4.5 は格子線を通らない
    let (artifact, _) = artifact(
        &[
            Point3D::new(-2.0, 4.0, -1.0),
            Point3D::new(22.0, 4.0, -1.0),
            Point3D::new(22.0, 5.0, -1.0),
            Point3D::new(-2.0, 5.0, -1.0),
        ],
        SegmentType::Cutting { feed_rate: 100.0 },
    );
    let surface = surface_at(&artifact, artifact.snapshots.len() - 1, PRECISION, RADIUS);
    // 球の中心の高さ 0.5 から、中心の線から 0.5 離れた稜線の高さ
    let ridge = 0.5 - (RADIUS * RADIUS - 0.25_f64).sqrt();
    let highest = all_vertices(&surface)
        .into_iter()
        .filter(|(position, _)| {
            (position[1] - 4.5).abs() < 0.125 && (5.0..15.0).contains(&position[0])
        })
        .map(|(position, _)| position[2])
        .fold(f64::NEG_INFINITY, f64::max);
    // 格子の辺の上の交点だけでは、稜線から最も近い格子線 y = 4.375 の高さ（約 0.038 低い）までしか届かない
    assert!(
        (highest - ridge).abs() < 0.01,
        "highest {highest}, ridge {ridge}"
    );
}

#[test]
fn uncut_stock_is_a_box_with_outward_normals() {
    let (artifact, _) = artifact(
        &[Point3D::new(0.0, 0.0, 5.0), Point3D::new(1.0, 0.0, 5.0)],
        SegmentType::Rapid,
    );
    let surface = surface_at(&artifact, 0, PRECISION, RADIUS);
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
    let surface = surface_at(&artifact, 0, PRECISION, RADIUS);
    let triangles = all_triangles(&surface);
    let [bx, by, bz] = surface.block_counts();
    assert_eq!(triangles.len(), 2 * 2 * (bx * by + bx * bz + by * bz));
    // 素材の角は面取りされない（角の頂点は、素材の面の座標軸の向きの法線の QEF でちょうど角に来る）
    let corner = [20.0, 10.0, 0.0];
    assert!(triangles.iter().flatten().any(|&point| point == corner));
}

#[test]
fn uncut_blocks_give_the_same_mesh_as_meshing_cell_by_cell() {
    let (artifact, _) = cut();
    for snapshot in [0, artifact.snapshots.len() - 1] {
        let surface = surface_at(&artifact, snapshot, PRECISION, RADIUS);
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
        let surface = surface_at(&artifact, artifact.snapshots.len() - 1, precision, RADIUS);
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
    let mut surface = surface_at(&artifact, 0, PRECISION, RADIUS);
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
        for block in surface.apply_changes(artifact.keyframe_changes.keyframe(snapshot)) {
            meshes.insert(block, surface.mesh_block(block));
        }
        let expected = surface_at(&artifact, snapshot, PRECISION, RADIUS);
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
        let surface = surface_at(&artifact, snapshot, PRECISION, RADIUS);
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
    let last = artifact.snapshots.len() - 1;
    let mut surface = surface_at(&artifact, last, PRECISION, RADIUS);
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
        // 時点 `snapshot` で変わったデクセルを、1 つ前の時点の区間に戻す
        let previous = artifact.dexel_states_at_keyframe(snapshot - 1);
        let restored: Vec<DexelState<'_>> = artifact
            .keyframe_changes
            .keyframe(snapshot)
            .map(|change| DexelState {
                intervals: &previous.intervals[change.grid as usize][change.index as usize],
                normals: &previous.normals[change.grid as usize][change.index as usize],
                ..change
            })
            .collect();
        for block in surface.apply_changes(restored) {
            meshes.insert(block, surface.mesh_block(block));
        }
        let expected = surface_at(&artifact, snapshot - 1, PRECISION, RADIUS);
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
    let surface = surface_at(&artifact, artifact.snapshots.len() - 1, PRECISION, RADIUS);
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
    let mut surface = surface_at(&artifact, 1, PRECISION, RADIUS);
    // 同じ時点の変化をもう一度当てても、区間は変わらない
    assert!(
        surface
            .apply_changes(artifact.keyframe_changes.keyframe(1))
            .is_empty()
    );
}
