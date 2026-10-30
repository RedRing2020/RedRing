//! `TriDexelModel` のテスト

use std::f64::consts::PI;

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};

use super::ray_interval::{AxisLine, swept_primitive_intervals};
use super::*;
use crate::stock_model::{SweptToolPrimitive, SweptToolShape};

const FEED: SegmentType<f64> = SegmentType::Cutting { feed_rate: 100.0 };

fn toolpath(tool_id: &str, points: &[Point3D<f64>]) -> ToolPath<f64> {
    ToolPath::new(
        tool_id.to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            points
                .windows(2)
                .map(|pair| PathSegment::new_line(pair[0], pair[1], FEED))
                .collect(),
        )],
        vec![],
    )
}

/// 決定的な擬似乱数（0 以上 1 未満）
fn random_sequence(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1_u64 << 53) as f64
    }
}

/// 区間の和を、昇順で互いに素な区間の列にする
fn merged(mut intervals: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut result: Vec<(f64, f64)> = Vec::new();
    for (low, high) in intervals {
        match result.last_mut() {
            Some(last) if low <= last.1 => last.1 = last.1.max(high),
            _ => result.push((low, high)),
        }
    }
    result
}

fn point_on(line: AxisLine, s: f64) -> Point3D<f64> {
    let mut coordinates = [line.point.x(), line.point.y(), line.point.z()];
    coordinates[line.axis] = s;
    Point3D::new(coordinates[0], coordinates[1], coordinates[2])
}

#[test]
fn line_intervals_match_the_closed_form_containment_of_the_swept_volume() {
    let mut next = random_sequence(7);
    for shape in [
        SweptToolShape::FlatEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
        SweptToolShape::BallEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
    ] {
        for (start, end) in [
            (Point3D::new(-3.0, 1.0, 4.0), Point3D::new(5.0, -2.0, -1.0)),
            (Point3D::new(0.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)),
            (Point3D::new(1.0, 1.0, 5.0), Point3D::new(1.0, 1.0, 0.0)),
        ] {
            let primitive = SweptToolPrimitive { start, end, shape };
            for _ in 0..300 {
                let line = AxisLine {
                    axis: (next() * 3.0) as usize,
                    point: Point3D::new(
                        -6.0 + 14.0 * next(),
                        -5.0 + 9.0 * next(),
                        -3.0 + 14.0 * next(),
                    ),
                };
                let intervals: Vec<(f64, f64)> = swept_primitive_intervals(&primitive, line)
                    .into_iter()
                    .flatten()
                    .map(|interval| interval.range)
                    .collect();
                for i in 0..=400 {
                    let s = -8.0 + 0.04 * f64::from(i);
                    let inside = intervals
                        .iter()
                        .any(|&(low, high)| (low..=high).contains(&s));
                    let near_end = intervals
                        .iter()
                        .any(|&(low, high)| (s - low).abs() < 1e-9 || (s - high).abs() < 1e-9);
                    assert!(
                        near_end || inside == primitive.contains(point_on(line, s)),
                        "{shape:?} {line:?} s = {s}"
                    );
                }
                // 結合した区間の端は掃引体の境界上にある（ボールは球と円柱の 2 つの区間が重なる）
                for (low, high) in merged(intervals) {
                    assert!(primitive.contains(point_on(line, low + 1e-9)) || high - low < 1e-8);
                    assert!(!primitive.contains(point_on(line, low - 1e-7)));
                    assert!(!primitive.contains(point_on(line, high + 1e-7)));
                }
            }
        }
    }
}

/// 素材 (-10, -10, -6)–(10, 10, 0) を、ボールエンドミルで斜めに掘り下げてから水平に削る経路
fn sample_model() -> (StockModel, Tool<f64>, Vec<Point3D<f64>>) {
    let tool = Tool::ball_end_mill("BEM3".to_string(), 3.0, 8.0);
    let points = vec![
        Point3D::new(-8.0, -6.0, 1.0),
        Point3D::new(-2.0, -1.0, -3.0),
        Point3D::new(4.0, 2.0, -3.0),
        Point3D::new(6.0, 7.0, -1.5),
    ];
    let stock = Aabb3D::new(
        Point3D::new(-10.0, -10.0, -6.0),
        Point3D::new(10.0, 10.0, 0.0),
    );
    let model = StockModel::from_toolpath(stock, &toolpath("BEM3", &points), &tool).unwrap();
    (model, tool, points)
}

/// X・Y・Z の格子の、デクセルごとの区間
type DexelStates = [Vec<Vec<(f64, f64)>>; 3];

/// 各スナップショットの時点の区間（初期の区間に、時点ごとの変化を順に当てる）
fn states_at_each_snapshot(dexels: &TriDexelModel) -> Vec<DexelStates> {
    let mut state = dexels
        .grids
        .each_ref()
        .map(|grid| vec![vec![grid.initial]; grid.slots.len()]);
    let changes = dexels.snapshot_changes();
    (0..changes.snapshot_count())
        .map(|snapshot| {
            for change in changes.snapshot(snapshot) {
                state[change.grid as usize][change.index as usize] = change.intervals.to_vec();
            }
            state.clone()
        })
        .collect()
}

#[test]
fn dexel_intervals_agree_with_the_stock_model() {
    let (model, _, _) = sample_model();
    let dexels = TriDexelModel::from_stock_model(&model, 0.5, &[model.primitives().len()]);
    for (grid_index, grid) in dexels.grids.iter().enumerate() {
        for i in 0..grid.count[0] {
            for j in (0..grid.count[1]).step_by(3) {
                let line = grid.line(i, j);
                let intervals = dexels.current_intervals(grid_index, i * grid.count[1] + j);
                let [low, high] = [
                    [
                        model.stock().min().x(),
                        model.stock().min().y(),
                        model.stock().min().z(),
                    ][grid.axis],
                    [
                        model.stock().max().x(),
                        model.stock().max().y(),
                        model.stock().max().z(),
                    ][grid.axis],
                ];
                for k in 0..=200 {
                    let s = low + (high - low) * f64::from(k) / 200.0;
                    let inside = intervals.iter().any(|&(a, b)| (a..=b).contains(&s));
                    let near_end = intervals
                        .iter()
                        .any(|&(a, b)| (s - a).abs() < 1e-9 || (s - b).abs() < 1e-9);
                    assert!(
                        near_end || inside == model.contains_material(point_on(line, s)),
                        "axis {} line {line:?} s = {s}",
                        grid.axis
                    );
                }
            }
        }
    }
}

#[test]
fn restored_states_match_models_built_up_to_each_time() {
    let (model, tool, points) = sample_model();
    // 同じ時点が続く場合と、掃引体を飛ばす場合を含める
    let applied = [0, 0, 1, 3];
    let dexels = TriDexelModel::from_stock_model(&model, 0.5, &applied);
    let states = states_at_each_snapshot(&dexels);
    assert_eq!(states.len(), applied.len());
    // 同じ時点が続くスナップショットには変化がない
    assert_eq!(dexels.snapshot_changes().snapshot(1).len(), 0);
    for (state, &applied) in states.iter().zip(&applied) {
        let partial = if applied == 0 {
            StockModel::from_toolpath(model.stock(), &toolpath("BEM3", &[]), &tool).unwrap()
        } else {
            StockModel::from_toolpath(model.stock(), &toolpath("BEM3", &points[..=applied]), &tool)
                .unwrap()
        };
        let expected =
            TriDexelModel::from_stock_model(&partial, 0.5, &[partial.primitives().len()]);
        for (grid, restored_grid) in state.iter().enumerate() {
            for (index, intervals) in restored_grid.iter().enumerate() {
                assert_eq!(
                    intervals.as_slice(),
                    expected.current_intervals(grid, index),
                    "applied = {applied}"
                );
            }
        }
    }
}

#[test]
fn dexels_relocated_to_hold_more_intervals_keep_the_other_dexels() {
    let stock = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 1.0, 1.0));
    let mut grid = DexelGrid::new(&stock, 2, 1.0);
    assert_eq!(grid.slots.len(), 2);
    let split = [(0.0, 0.2), (0.4, 0.6), (0.8, 1.0)];
    let normals = [0, 1, 2].map(initial_normals);
    grid.set(0, &split, &normals);
    grid.set(0, &split[..1], &normals[..1]);
    grid.set(1, &[(0.5, 1.0)], &normals[2..]);
    assert_eq!(grid.current(0), &split[..1]);
    assert_eq!(grid.current_normals(0), &normals[..1]);
    assert_eq!(grid.current(1), &[(0.5, 1.0)]);
    assert_eq!(grid.current_normals(1), &normals[2..]);
    grid.set(0, &split, &normals);
    assert_eq!(grid.current(0), &split);
    assert_eq!(grid.current_normals(0), &normals);
}

#[test]
fn accumulated_remaining_volumes_match_the_intervals_at_each_time() {
    let (model, _, _) = sample_model();
    let applied: Vec<usize> = (0..=model.primitives().len()).collect();
    let dexels = TriDexelModel::from_stock_model(&model, 0.5, &applied);
    let grid = &dexels.grids[2];
    for (state, &applied) in states_at_each_snapshot(&dexels).iter().zip(&applied) {
        let length: f64 = state[2]
            .iter()
            .flatten()
            .map(|(low, high)| high - low)
            .sum();
        let expected = length * grid.pitch[0] * grid.pitch[1];
        let remaining = dexels.remaining_volume_at(applied);
        assert!(
            (remaining - expected).abs() < 1e-9 * expected,
            "applied = {applied}: {remaining} != {expected}"
        );
    }
}

#[test]
fn remaining_volume_of_a_flat_slot() {
    // 深さ 2 の溝: 直径 4 のフラットエンドミルの先端が (0, 0, -2) から (10, 0, -2) へ移動する
    let tool = Tool::flat_end_mill("EM4".to_string(), 4.0, 10.0);
    let stock = Aabb3D::new(Point3D::new(-5.0, -5.0, -5.0), Point3D::new(15.0, 5.0, 0.0));
    let path = toolpath(
        "EM4",
        &[Point3D::new(0.0, 0.0, -2.0), Point3D::new(10.0, 0.0, -2.0)],
    );
    let model = StockModel::from_toolpath(stock, &path, &tool).unwrap();
    let dexels = TriDexelModel::from_stock_model(&model, 0.05, &[0, 1]);

    let initial = 20.0 * 10.0 * 5.0;
    let removed = (4.0 * 10.0 + PI * 4.0) * 2.0;
    assert!((dexels.remaining_volume_at(0) - initial).abs() < 1e-9);
    let remaining = dexels.remaining_volume_at(1);
    assert!(
        (remaining - (initial - removed)).abs() < 0.01 * removed,
        "remaining = {remaining}"
    );
}

#[test]
fn subtracting_intervals() {
    let intervals = [(0.0, 2.0), (3.0, 5.0)];
    let normals = [initial_normals(2); 2];
    let line = AxisLine {
        axis: 2,
        point: Point3D::new(0.0, 0.0, 0.0),
    };
    let subtracted = |range| {
        let removed = ToolInterval::cylinder_caps_for_test(range);
        let (mut result, mut result_normals) = (Vec::new(), Vec::new());
        subtract_interval(
            &intervals,
            &normals,
            &removed,
            line,
            &mut result,
            &mut result_normals,
        );
        assert_eq!(result.len(), result_normals.len());
        result
    };
    assert_eq!(subtracted((1.0, 4.0)), vec![(0.0, 1.0), (4.0, 5.0)]);
    assert_eq!(subtracted((-1.0, 6.0)), vec![]);
    assert_eq!(subtracted((2.0, 3.0)), intervals.to_vec());
    assert_eq!(
        subtracted((0.5, 1.5)),
        vec![(0.0, 0.5), (1.5, 2.0), (3.0, 5.0)]
    );
}

#[test]
fn subtracting_gives_the_cut_ends_the_reversed_tool_normals() {
    // 先端が (0, 0, 0.5) で止まっているフラットエンドミル（半径 1、長さ 1）を、Z の向きの線 (0.6, 0) で差し引く
    let primitive = SweptToolPrimitive {
        start: Point3D::new(0.0, 0.0, 0.5),
        end: Point3D::new(0.0, 0.0, 0.5),
        shape: SweptToolShape::FlatEndMill {
            radius: 1.0,
            cutting_length: 1.0,
        },
    };
    let line = AxisLine {
        axis: 2,
        point: Point3D::new(0.6, 0.0, 0.0),
    };
    let [removed, _] = swept_primitive_intervals(&primitive, line);
    let removed = removed.unwrap();
    assert_eq!(removed.range, (0.5, 1.5));
    let [bottom, top] = initial_normals(2);
    let (mut result, mut normals) = (Vec::new(), Vec::new());
    subtract_interval(
        &[(0.0, 2.0)],
        &[[bottom, top]],
        &removed,
        line,
        &mut result,
        &mut normals,
    );
    assert_eq!(result, vec![(0.0, 0.5), (1.5, 2.0)]);
    // 工具の底（-Z）の下の材料の上端は +Z、工具の上面（+Z）の上の材料の下端は -Z
    assert_eq!(normals, vec![[bottom, top], [bottom, top]]);
}

/// 線分の上の点 `point` に最も近い、`start`–`end` の上の点
fn closest_on_segment(point: [f64; 3], start: [f64; 3], end: [f64; 3]) -> [f64; 3] {
    let direction: [f64; 3] = std::array::from_fn(|axis| end[axis] - start[axis]);
    let length2: f64 = direction.iter().map(|d| d * d).sum();
    let t = if length2 == 0.0 {
        0.0
    } else {
        let along: f64 = (0..3)
            .map(|axis| (point[axis] - start[axis]) * direction[axis])
            .sum();
        (along / length2).clamp(0.0, 1.0)
    };
    std::array::from_fn(|axis| start[axis] + t * direction[axis])
}

fn coordinates_of(point: Point3D<f64>) -> [f64; 3] {
    [point.x(), point.y(), point.z()]
}

#[test]
fn tool_normals_point_out_of_the_swept_volume_at_the_interval_ends() {
    let mut next = random_sequence(11);
    for shape in [
        SweptToolShape::FlatEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
        SweptToolShape::BallEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
    ] {
        for (start, end) in [
            (Point3D::new(-3.0, 1.0, 4.0), Point3D::new(5.0, -2.0, -1.0)),
            (Point3D::new(0.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)),
            (Point3D::new(1.0, 1.0, 5.0), Point3D::new(1.0, 1.0, 0.0)),
            (Point3D::new(-2.0, 0.0, 0.0), Point3D::new(3.0, 2.0, 2.0)),
        ] {
            let primitive = SweptToolPrimitive { start, end, shape };
            for _ in 0..300 {
                let line = AxisLine {
                    axis: (next() * 3.0) as usize,
                    point: Point3D::new(
                        -6.0 + 14.0 * next(),
                        -5.0 + 9.0 * next(),
                        -3.0 + 14.0 * next(),
                    ),
                };
                let intervals: Vec<ToolInterval> = swept_primitive_intervals(&primitive, line)
                    .into_iter()
                    .flatten()
                    .collect();
                for interval in &intervals {
                    let ends = [interval.range.0, interval.range.1];
                    for (end, value) in ends.into_iter().enumerate() {
                        let normal = interval.normal(end, line);
                        let length: f64 = normal.iter().map(|c| c * c).sum::<f64>().sqrt();
                        assert!((length - 1.0).abs() < 1e-12, "{shape:?} {line:?}");
                        // 始まりの法線は線の負の向き、終わりは正の向きを向く
                        let sign = if end == 0 { -1.0 } else { 1.0 };
                        assert!(
                            sign * normal[line.axis] >= -1e-12,
                            "{shape:?} {line:?} {normal:?}"
                        );
                        // 法線の向きへ少し進むと掃引体の外、逆へ少し進むと中（他の凸な形が覆う端は外を調べない）
                        let point = coordinates_of(point_on(line, value));
                        let moved = |step: f64| {
                            Point3D::new(
                                point[0] + step * normal[0],
                                point[1] + step * normal[1],
                                point[2] + step * normal[2],
                            )
                        };
                        let covered = intervals.iter().any(|other| {
                            other != interval
                                && other.range.0 < value - 1e-6
                                && value + 1e-6 < other.range.1
                        });
                        if !covered && interval.range.1 - interval.range.0 > 1e-6 {
                            assert!(
                                !primitive.contains(moved(1e-6)),
                                "{shape:?} {line:?} end {end} {normal:?}"
                            );
                        }
                        assert!(
                            primitive.contains(moved(-1e-6))
                                || interval.range.1 - interval.range.0 < 1e-5,
                            "{shape:?} {line:?} end {end} {normal:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn ball_end_mill_cut_normals_point_to_the_sphere_center_path() {
    // 素材 (0, 0, -4)–(20, 10, 0) を、ボールエンドミル（半径 3）で深さ 2 の水平の溝を削る
    let tool = Tool::ball_end_mill("BEM3".to_string(), 6.0, 8.0);
    let points = [Point3D::new(2.0, 5.0, -2.0), Point3D::new(18.0, 5.0, -2.0)];
    let stock = Aabb3D::new(Point3D::new(0.0, 0.0, -4.0), Point3D::new(20.0, 10.0, 0.0));
    let model = StockModel::from_toolpath(stock, &toolpath("BEM3", &points), &tool).unwrap();
    let dexels = TriDexelModel::from_stock_model(&model, 0.5, &[model.primitives().len()]);
    let grid = &dexels.grids[2];
    let (center_start, center_end) = ([2.0, 5.0, 1.0], [18.0, 5.0, 1.0]);
    let mut checked = 0;
    for i in 0..grid.count[0] {
        for j in 0..grid.count[1] {
            let index = i * grid.count[1] + j;
            let line = grid.line(i, j);
            for (&(_, high), &[_, high_normal]) in dexels
                .current_intervals(2, index)
                .iter()
                .zip(dexels.current_normals(2, index))
            {
                if high >= 0.0 {
                    // 削っていない上面の法線は +Z
                    assert_eq!(high_normal, initial_normals(2)[1]);
                    continue;
                }
                // 球の中心の通り道から、削った面の点への向きの逆が工具の法線（材料の法線はその逆）
                let point = coordinates_of(point_on(line, high));
                let center = closest_on_segment(point, center_start, center_end);
                let expected: [f64; 3] =
                    std::array::from_fn(|axis| (center[axis] - point[axis]) / 3.0);
                let actual = high_normal.to_unit();
                let dot: f64 = expected.iter().zip(actual).map(|(a, b)| a * b).sum();
                assert!(dot > 1.0 - 1e-6, "{line:?} {expected:?} {actual:?}");
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}

#[test]
fn flat_end_mill_floor_and_wall_normals_are_axis_aligned() {
    // 素材 (0, 0, -4)–(20, 10, 0) を、フラットエンドミル（半径 2）で深さ 2 の水平の溝を削る
    let tool = Tool::flat_end_mill("FEM4".to_string(), 4.0, 8.0);
    let points = [Point3D::new(-5.0, 5.0, -2.0), Point3D::new(25.0, 5.0, -2.0)];
    let stock = Aabb3D::new(Point3D::new(0.0, 0.0, -4.0), Point3D::new(20.0, 10.0, 0.0));
    let model = StockModel::from_toolpath(stock, &toolpath("FEM4", &points), &tool).unwrap();
    let dexels = TriDexelModel::from_stock_model(&model, 0.5, &[model.primitives().len()]);
    // Z の向きのデクセル: 溝の底の上端の法線は +Z
    let z_grid = &dexels.grids[2];
    let mut floors = 0;
    for index in 0..z_grid.slots.len() {
        for (&(_, high), &[_, high_normal]) in dexels
            .current_intervals(2, index)
            .iter()
            .zip(dexels.current_normals(2, index))
        {
            if (high + 2.0).abs() < 1e-12 {
                assert_eq!(high_normal, DexelNormal::from_unit([0.0, 0.0, 1.0]));
                floors += 1;
            }
        }
    }
    assert!(floors > 0);
    // Y の向きのデクセル: 溝の壁（y = 3・7）の法線は、溝の側（壁から溝の中心へ）を向く
    let y_grid = &dexels.grids[1];
    let mut walls = 0;
    for index in 0..y_grid.slots.len() {
        for (&(low, high), &[low_normal, high_normal]) in dexels
            .current_intervals(1, index)
            .iter()
            .zip(dexels.current_normals(1, index))
        {
            if (high - 3.0).abs() < 1e-12 {
                assert_eq!(high_normal, DexelNormal::from_unit([0.0, 1.0, 0.0]));
                walls += 1;
            }
            if (low - 7.0).abs() < 1e-12 {
                assert_eq!(low_normal, DexelNormal::from_unit([0.0, -1.0, 0.0]));
                walls += 1;
            }
        }
    }
    assert!(walls > 0);
}
