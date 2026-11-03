//! CAM向けサンプルデータ生成。

use geo_algorithms::Point3D;

use crate::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath,
};

/// デバッグ用：サンプル工具経路を生成。
///
/// シンプルな矩形加工経路を返します（UI表示テスト用）。
pub fn create_sample_toolpath() -> ToolPath<f64> {
    // エアカット高さ（切削パスより +20）
    let aircut_z = 20.0;
    // contour1 コーナー円弧半径（R=8mm で視認可能な丸み）
    let r = 8.0_f64;

    // 開始位置から切削開始点までのRapid移動（XY平面で視認可能）
    let rapid_to_start = PathSegment::new_line(
        Point3D::new(0.0, 0.0, aircut_z),
        Point3D::new(-40.0 + r, -40.0, aircut_z),
        SegmentType::Rapid,
    );

    // アプローチセグメント: Z下降（切削面へ）
    let approach = PathSegment::new_line(
        Point3D::new(-40.0 + r, -40.0, aircut_z),
        Point3D::new(-40.0 + r, -40.0, 0.0),
        SegmentType::Approach { feed_rate: 300.0 },
    );

    // 最初の等高線レベル（Z = 0.0）- 外側周回（コーナーが円弧）
    let mut contour1_segments = vec![
        // 下辺
        PathSegment::new_line(
            Point3D::new(-40.0 + r, -40.0, 0.0),
            Point3D::new(40.0 - r, -40.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // コーナー (40,-40): CCW, center=(40-r, -40+r)
        PathSegment::new_arc(
            Point3D::new(40.0 - r, -40.0, 0.0),
            Point3D::new(40.0, -40.0 + r, 0.0),
            Point3D::new(40.0 - r, -40.0 + r, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // 右辺
        PathSegment::new_line(
            Point3D::new(40.0, -40.0 + r, 0.0),
            Point3D::new(40.0, 40.0 - r, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // コーナー (40,40): CCW, center=(40-r, 40-r)
        PathSegment::new_arc(
            Point3D::new(40.0, 40.0 - r, 0.0),
            Point3D::new(40.0 - r, 40.0, 0.0),
            Point3D::new(40.0 - r, 40.0 - r, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // 上辺
        PathSegment::new_line(
            Point3D::new(40.0 - r, 40.0, 0.0),
            Point3D::new(-40.0 + r, 40.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // コーナー (-40,40): CCW, center=(-40+r, 40-r)
        PathSegment::new_arc(
            Point3D::new(-40.0 + r, 40.0, 0.0),
            Point3D::new(-40.0, 40.0 - r, 0.0),
            Point3D::new(-40.0 + r, 40.0 - r, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // 左辺
        PathSegment::new_line(
            Point3D::new(-40.0, 40.0 - r, 0.0),
            Point3D::new(-40.0, -40.0 + r, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        // コーナー (-40,-40): CCW, center=(-40+r, -40+r)
        PathSegment::new_arc(
            Point3D::new(-40.0, -40.0 + r, 0.0),
            Point3D::new(-40.0 + r, -40.0, 0.0),
            Point3D::new(-40.0 + r, -40.0 + r, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
    ];

    // 周回間リトラクト1: 内側周回へ直接移動（Z=0のまま斜め移動）
    let pass_retract_1 = PathSegment::new_line(
        Point3D::new(-40.0 + r, -40.0, 0.0),
        Point3D::new(-35.0, -35.0, 0.0),
        SegmentType::PassRetract { feed_rate: 300.0 },
    );
    contour1_segments.push(pass_retract_1);
    let contour1 = ContourLevelPath::new(0, 0.0, contour1_segments);

    // 1層目内側周回（10mm短い正方形）
    let mut contour1_inner_segments = vec![
        PathSegment::new_line(
            Point3D::new(-35.0, -35.0, 0.0),
            Point3D::new(35.0, -35.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(35.0, -35.0, 0.0),
            Point3D::new(35.0, 35.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(35.0, 35.0, 0.0),
            Point3D::new(-35.0, 35.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(-35.0, 35.0, 0.0),
            Point3D::new(-35.0, -35.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
    ];

    // 1層目終了後のリトラクト: Z=10まで上昇
    let retract_1 = PathSegment::new_line(
        Point3D::new(-35.0, -35.0, 0.0),
        Point3D::new(-35.0, -35.0, 10.0),
        SegmentType::Retract { feed_rate: 300.0 },
    );

    // 周回間Rapid2: 2層目始点へ移動（Z=10のまま）
    let pass_rapid_2 = PathSegment::new_line(
        Point3D::new(-35.0, -35.0, 10.0),
        Point3D::new(-40.0, -40.0, 10.0),
        SegmentType::Rapid,
    );

    // 周回間アプローチ2: 2層目へ下降
    let pass_approach_2 = PathSegment::new_line(
        Point3D::new(-40.0, -40.0, 10.0),
        Point3D::new(-40.0, -40.0, -5.0),
        SegmentType::Approach { feed_rate: 300.0 },
    );

    contour1_inner_segments.push(retract_1);
    contour1_inner_segments.push(pass_rapid_2);
    contour1_inner_segments.push(pass_approach_2);
    let contour1_inner = ContourLevelPath::new(1, 0.0, contour1_inner_segments);

    // 次の等高線レベル（Z = -5.0）
    let contour2_segments = vec![
        PathSegment::new_line(
            Point3D::new(-40.0, -40.0, -5.0),
            Point3D::new(40.0, -40.0, -5.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(40.0, -40.0, -5.0),
            Point3D::new(40.0, 40.0, -5.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(40.0, 40.0, -5.0),
            Point3D::new(-40.0, 40.0, -5.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
        PathSegment::new_line(
            Point3D::new(-40.0, 40.0, -5.0),
            Point3D::new(-40.0, -40.0, -5.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        ),
    ];
    let contour2 = ContourLevelPath::new(2, -5.0, contour2_segments);

    // 最終リトラクト: Z上昇
    let final_retract = PathSegment::new_line(
        Point3D::new(-40.0, -40.0, -5.0),
        Point3D::new(-40.0, -40.0, aircut_z),
        SegmentType::Retract { feed_rate: 300.0 },
    );

    // 終了位置へのRapid移動（XY平面で視認可能）
    let rapid_to_end = PathSegment::new_line(
        Point3D::new(-40.0, -40.0, aircut_z),
        Point3D::new(0.0, 0.0, aircut_z),
        SegmentType::Rapid,
    );

    ToolPath::new(
        "endmill_3mm".to_string(),
        CuttingDirection::Down,
        vec![rapid_to_start, approach],
        vec![contour1, contour1_inner, contour2],
        vec![final_retract, rapid_to_end],
    )
}

/// デバッグ用：空の工具経路を生成する。
pub fn create_empty_toolpath() -> ToolPath<f64> {
    ToolPath::new(
        "endmill_3mm".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![],
        vec![],
    )
}

/// デバッグ用：ボールエンドミル工具を生成する。
pub fn create_sample_ball_end_mill_tool() -> Tool<f64> {
    Tool::ball_end_mill("ball_endmill_10mm".to_string(), 10.0, 50.0)
}

/// デバッグ用：フラットエンドミル工具を生成する。
pub fn create_sample_flat_end_mill_tool() -> Tool<f64> {
    Tool::flat_end_mill("flat_endmill_10mm".to_string(), 10.0, 50.0)
}
