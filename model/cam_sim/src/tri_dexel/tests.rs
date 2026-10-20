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
                let intervals = swept_primitive_intervals(&primitive, line);
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

#[test]
fn dexel_intervals_agree_with_the_stock_model() {
    let (model, _, _) = sample_model();
    let dexels = TriDexelModel::from_stock_model(&model, 0.5);
    for grid in &dexels.grids {
        for i in 0..grid.count[0] {
            for j in (0..grid.count[1]).step_by(3) {
                let line = grid.line(i, j);
                let intervals =
                    grid.dexels[i * grid.count[1] + j].intervals_at(model.primitives().len());
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
    let dexels = TriDexelModel::from_stock_model(&model, 0.5);
    assert_eq!(dexels.primitive_count(), points.len() - 1);
    for applied in 0..points.len() {
        let partial = if applied == 0 {
            StockModel::from_toolpath(model.stock(), &toolpath("BEM3", &[]), &tool).unwrap()
        } else {
            StockModel::from_toolpath(model.stock(), &toolpath("BEM3", &points[..=applied]), &tool)
                .unwrap()
        };
        let expected = TriDexelModel::from_stock_model(&partial, 0.5);
        for (grid, expected_grid) in dexels.grids.iter().zip(&expected.grids) {
            for (dexel, expected_dexel) in grid.dexels.iter().zip(&expected_grid.dexels) {
                assert_eq!(
                    dexel.intervals_at(applied),
                    expected_dexel.intervals_at(applied),
                    "applied = {applied}"
                );
            }
        }
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
    let dexels = TriDexelModel::from_stock_model(&model, 0.05);

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
    assert_eq!(
        subtract_interval(&intervals, (1.0, 4.0)),
        vec![(0.0, 1.0), (4.0, 5.0)]
    );
    assert_eq!(subtract_interval(&intervals, (-1.0, 6.0)), vec![]);
    assert_eq!(
        subtract_interval(&intervals, (2.0, 3.0)),
        intervals.to_vec()
    );
    assert_eq!(
        subtract_interval(&intervals, (0.5, 1.5)),
        vec![(0.0, 0.5), (1.5, 2.0), (3.0, 5.0)]
    );
}
