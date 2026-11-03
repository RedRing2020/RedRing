//! `StockModel` のテスト

use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};

use super::*;

const FEED: SegmentType<f64> = SegmentType::Cutting { feed_rate: 100.0 };

fn toolpath(
    tool_id: &str,
    segments: &[(Point3D<f64>, Point3D<f64>, SegmentType<f64>)],
) -> ToolPath<f64> {
    ToolPath::new(
        tool_id.to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            segments
                .iter()
                .map(|&(start, end, kind)| PathSegment::new_line(start, end, kind))
                .collect(),
        )],
        vec![],
    )
}

fn stock() -> Aabb3D<f64> {
    Aabb3D::new(
        Point3D::new(-20.0, -20.0, -20.0),
        Point3D::new(20.0, 20.0, 20.0),
    )
}

/// 直径 4・刃長 10 の工具で、先端が (0, 0, 0) から (10, 0, 0) へ水平に移動した正本
fn horizontal_cut(tool: &Tool<f64>) -> StockModel {
    let path = toolpath(
        &tool.id,
        &[(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            FEED,
        )],
    );
    StockModel::from_toolpath(stock(), &path, tool).unwrap()
}

#[test]
fn flat_end_mill_sweeps_a_vertical_cylinder_with_a_flat_bottom() {
    let model = horizontal_cut(&Tool::flat_end_mill("EM4".to_string(), 4.0, 10.0));
    for (point, removed) in [
        // 底面（先端の高さ）は除去し、その下は残る
        (Point3D::new(5.0, 0.0, 0.0), true),
        (Point3D::new(5.0, 0.0, -0.01), false),
        // 側面は工具の半径まで。経路を軸とする横向きの円柱なら残る点も除去する
        (Point3D::new(5.0, 1.9, 1.9), true),
        (Point3D::new(5.0, 2.1, 1.0), false),
        // 刃長より上は除去しない
        (Point3D::new(5.0, 0.0, 9.9), true),
        (Point3D::new(5.0, 0.0, 10.1), false),
        // 移動の両端の円柱
        (Point3D::new(-1.9, 0.0, 1.0), true),
        (Point3D::new(11.9, 0.0, 1.0), true),
        (Point3D::new(12.1, 0.0, 1.0), false),
    ] {
        assert_eq!(model.contains_material(point), !removed, "{point:?}");
    }
}

#[test]
fn a_tool_without_a_length_removes_up_to_the_top_of_the_stock() {
    for mut tool in [
        Tool::flat_end_mill("EM4".to_string(), 4.0, 10.0),
        Tool::ball_end_mill("BEM4".to_string(), 4.0, 10.0),
    ] {
        tool.cutting_length = None;
        let model = horizontal_cut(&tool);
        // 工具長 10 より上も、素材の上面（z = 20）まで除去する
        for z in [10.1, 19.9, 20.0] {
            assert!(!model.contains_material(Point3D::new(5.0, 0.0, z)), "{z}");
        }
        // 工具の半径の外側は残る
        assert!(model.contains_material(Point3D::new(5.0, 2.1, 15.0)));
    }
}

#[test]
fn ball_end_mill_sweeps_a_ball_and_a_cylinder_above_its_center() {
    let model = horizontal_cut(&Tool::ball_end_mill("BEM4".to_string(), 4.0, 10.0));
    for (point, removed) in [
        // 球の最下点（先端）と、中心の高さの側面
        (Point3D::new(5.0, 0.0, 0.05), true),
        (Point3D::new(5.0, 1.9, 2.0), true),
        // 球の外側で中心より下はフラットと異なり残る
        (Point3D::new(5.0, 1.9, 0.1), false),
        // 中心より上は円柱
        (Point3D::new(5.0, 1.9, 5.0), true),
        (Point3D::new(5.0, 0.0, 10.1), false),
    ] {
        assert_eq!(model.contains_material(point), !removed, "{point:?}");
    }
}

#[test]
fn plunge_removes_the_tool_body_over_the_whole_depth() {
    // 先端が (0, 0, 5) から (0, 0, 0) へ下降する
    let tool = Tool::flat_end_mill("EM4".to_string(), 4.0, 10.0);
    let path = toolpath(
        "EM4",
        &[(
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(0.0, 0.0, 0.0),
            FEED,
        )],
    );
    let model = StockModel::from_toolpath(stock(), &path, &tool).unwrap();
    assert!(!model.contains_material(Point3D::new(0.0, 0.0, 1.0)));
    assert!(!model.contains_material(Point3D::new(1.5, 0.0, 14.5)));
    assert!(model.contains_material(Point3D::new(0.0, 0.0, 15.5)));
    assert!(model.contains_material(Point3D::new(0.0, 0.0, -0.5)));
}

/// 細かく分けた工具の位置のいずれかに含まれるか（閉じた式の判定との比較用）
fn sampled_contains(primitive: &SweptToolPrimitive, point: Point3D<f64>) -> bool {
    let direction = primitive.direction();
    (0..=4_000).any(|i| {
        let t = f64::from(i) / 4_000.0;
        let tip = Point3D::new(
            primitive.start.x() + t * direction[0],
            primitive.start.y() + t * direction[1],
            primitive.start.z() + t * direction[2],
        );
        let at_tip = SweptToolPrimitive {
            start: tip,
            end: tip,
            shape: primitive.shape,
        };
        at_tip.contains(point)
    })
}

#[test]
fn swept_volume_matches_tool_positions_sampled_along_an_oblique_move() {
    let shapes = [
        SweptToolShape::FlatEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
        SweptToolShape::BallEndMill {
            radius: 2.0,
            cutting_length: 6.0,
        },
    ];
    for shape in shapes {
        let primitive = SweptToolPrimitive {
            start: Point3D::new(-3.0, 1.0, 4.0),
            end: Point3D::new(5.0, -2.0, -1.0),
            shape,
        };
        // 決定的な擬似乱数の点で、境界から離れた点の判定が一致する
        let mut state = 12_345_u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (state >> 11) as f64 / (1_u64 << 53) as f64
        };
        for _ in 0..500 {
            let point = Point3D::new(
                -6.0 + 14.0 * next(),
                -6.0 + 10.0 * next(),
                -4.0 + 14.0 * next(),
            );
            let exact = primitive.contains(point);
            let sampled = sampled_contains(&primitive, point);
            // 分けた位置の工具は掃引体に含まれる
            assert!(!sampled || exact, "{shape:?} {point:?}");
            // 分けた位置の間で取りこぼすのは、掃引体の境界のごく近くの点に限る
            if exact && !sampled {
                let near_boundary = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
                    .iter()
                    .flat_map(|axis| [1e-3, -1e-3].map(|step| axis.map(|c| c * step)))
                    .any(|[dx, dy, dz]| {
                        !primitive.contains(Point3D::new(
                            point.x() + dx,
                            point.y() + dy,
                            point.z() + dz,
                        ))
                    });
                assert!(near_boundary, "{shape:?} {point:?}");
            }
        }
    }
}

#[test]
fn stock_bounds_and_non_cutting_segments() {
    let tool = Tool::flat_end_mill("EM4".to_string(), 4.0, 10.0);
    let path = toolpath(
        "EM4",
        &[(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            SegmentType::Rapid,
        )],
    );
    let model = StockModel::from_toolpath(stock(), &path, &tool).unwrap();
    // 早送りでは除去しない（素材への食い込みは別の確認で扱う）
    assert!(model.contains_material(Point3D::new(5.0, 0.0, 1.0)));
    // 素材の外は材料ではない（境界は素材に含める）
    assert!(model.contains_material(Point3D::new(20.0, 0.0, 0.0)));
    assert!(!model.contains_material(Point3D::new(20.1, 0.0, 0.0)));
}

#[test]
fn radius_end_mill_is_not_supported_yet() {
    let tool = Tool::radius_end_mill("REM4".to_string(), 4.0, 0.5, 10.0);
    let path = toolpath(
        "REM4",
        &[(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            FEED,
        )],
    );
    assert!(matches!(
        StockModel::from_toolpath(stock(), &path, &tool),
        Err(SimulationError::UnsupportedToolType)
    ));
}

#[test]
fn hierarchy_finds_the_same_primitives_as_a_linear_scan_with_few_candidates() {
    // 100 × 100 本の短い往復の経路（1 万の掃引体）
    let tool = Tool::ball_end_mill("BEM1".to_string(), 1.0, 5.0);
    let mut segments = Vec::new();
    for row in 0..100 {
        let y = -10.0 + 0.2 * f64::from(row);
        for column in 0..100 {
            let x = -10.0 + 0.2 * f64::from(column);
            let z = -0.5 * (f64::from(row + column) * 0.07).sin();
            segments.push((Point3D::new(x, y, z), Point3D::new(x + 0.2, y, z), FEED));
        }
    }
    let model = StockModel::from_toolpath(stock(), &toolpath("BEM1", &segments), &tool).unwrap();
    assert_eq!(model.primitives.len(), 10_000);

    let mut most_candidates = 0;
    for i in 0..400 {
        let point = Point3D::new(
            -11.0 + 0.055 * f64::from(i),
            -11.0 + 0.05 * f64::from((i * 7) % 400),
            -1.0 + 0.004 * f64::from((i * 13) % 400),
        );
        let linear = model
            .primitives
            .iter()
            .any(|primitive| primitive.contains(point));
        let mut candidates = 0;
        let found = model.hierarchy.any_containing(point, |index| {
            candidates += 1;
            model.primitives[index].contains(point)
        });
        assert_eq!(found, linear, "{point:?}");
        most_candidates = most_candidates.max(candidates);
    }
    // 包含箱が点を含む掃引体だけを調べる（工具の直径 2 の範囲の掃引体は 100 本程度）
    assert!(most_candidates < 500, "candidates = {most_candidates}");
}
