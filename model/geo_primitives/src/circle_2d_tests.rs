//! Circle2D のテスト

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::{
    core::{
        BasicMeasurement,
        // 新階層Foundation
        GeometryShape,
        NewBasicContainment as BasicContainment,
        NewBasicParametric,
        ParametricShape,
        Shape2D,
        SurfaceShape,
    },
    Scalar,
};
use std::f64::consts::{PI, TAU};

/// 基本作成テスト
#[test]
fn test_circle2d_creation() {
    let center = Point2D::new(2.0, 3.0);
    let radius = 5.0;
    let circle = Circle2D::new(center, radius).unwrap();

    assert_eq!(circle.center(), center);
    assert_eq!(circle.radius(), radius);
    assert_eq!(circle.diameter(), 10.0);
}

/// 無効な円の作成テスト
#[test]
fn test_circle2d_invalid_creation() {
    let center = Point2D::new(0.0, 0.0);

    // 負の半径
    assert!(Circle2D::new(center, -1.0).is_none());

    // ゼロ半径
    assert!(Circle2D::new(center, 0.0).is_none());
}

/// 単位円テスト
#[test]
fn test_unit_circle() {
    let circle = Circle2D::<f64>::unit_circle();

    assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
    assert_eq!(circle.radius(), 1.0);
    assert_eq!(circle.diameter(), 2.0);
}

/// 3点からの外接円テスト
#[test]
fn test_from_three_points() {
    // 直角三角形の3点
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(3.0, 0.0);
    let p3 = Point2D::new(0.0, 4.0);

    let circle = Circle2D::from_three_points(p1, p2, p3).unwrap();

    // 外接円の中心は(1.5, 2.0)、半径は2.5
    assert!((circle.center().x() - 1.5f64).abs() < 1e-10);
    assert!((circle.center().y() - 2.0f64).abs() < 1e-10);
    assert!((circle.radius() - 2.5f64).abs() < 1e-10);

    // 3点すべてが円上にあることを確認
    assert!(circle.contains_point_on_circle(&p1, 1e-10));
    assert!(circle.contains_point_on_circle(&p2, 1e-10));
    assert!(circle.contains_point_on_circle(&p3, 1e-10));
}

/// 共線点からの外接円テスト（失敗ケース）
#[test]
fn test_from_collinear_points() {
    let p1 = Point2D::new(0.0, 0.0);
    let p2 = Point2D::new(1.0, 1.0);
    let p3 = Point2D::new(2.0, 2.0);

    // 共線点からは円を作れない
    assert!(Circle2D::from_three_points(p1, p2, p3).is_none());
}

/// 円周・面積計算テスト
#[test]
fn test_circle2d_metrics() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

    // 円周 = 2πr
    let expected_circumference = TAU * 2.0;
    assert!((circle.circumference() - expected_circumference).abs() < 1e-10);

    // 面積 = πr²
    let expected_area = PI * 4.0;
    assert!((circle.area() - expected_area).abs() < 1e-10);
}

/// 角度での点取得テスト
#[test]
fn test_point_at_angle() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    // 0度の点（右方向）
    let p0 = circle.point_at_angle(0.0);
    assert!((p0.x() - 3.0f64).abs() < 1e-10);
    assert!((p0.y() - 1.0f64).abs() < 1e-10);

    // 90度の点（上方向）
    let p90 = circle.point_at_angle(PI / 2.0);
    assert!((p90.x() - 1.0).abs() < 1e-10);
    assert!((p90.y() - 3.0).abs() < 1e-10);

    // 180度の点（左方向）
    let p180 = circle.point_at_angle(PI);
    assert!((p180.x() - (-1.0)).abs() < 1e-10);
    assert!((p180.y() - 1.0).abs() < 1e-10);
}

/// パラメータでの点取得テスト
#[test]
fn test_point_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // t=0（開始点）
    let p0 = circle.point_at_parameter(0.0);
    assert!((p0.x() - 1.0f64).abs() < 1e-10);
    assert!((p0.y() - 0.0f64).abs() < 1e-10);

    // t=0.25（90度）
    let p25 = circle.point_at_parameter(0.25);
    assert!((p25.x() - 0.0f64).abs() < 1e-10);
    assert!((p25.y() - 1.0f64).abs() < 1e-10);

    // t=0.5（180度）
    let p50 = circle.point_at_parameter(0.5);
    assert!((p50.x() - (-1.0f64)).abs() < 1e-10);
    assert!((p50.y() - 0.0f64).abs() < 1e-10);

    // t=1.0（360度、開始点と同じ）
    let p100 = circle.point_at_parameter(1.0);
    assert!((p100.x() - 1.0f64).abs() < 1e-10);
    assert!((p100.y() - 0.0f64).abs() < 1e-10);
}

/// 接線ベクトルテスト
#[test]
fn test_tangent_at_parameter() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // t=0での接線（上方向）
    let t0 = circle.tangent_at_parameter(0.0);
    assert!((t0.x() - 0.0f64).abs() < 1e-10);
    assert!((t0.y() - 1.0f64).abs() < 1e-10);

    // t=0.25での接線（左方向）
    let t25 = circle.tangent_at_parameter(0.25);
    assert!((t25.x() - (-1.0f64)).abs() < 1e-10);
    assert!((t25.y() - 0.0f64).abs() < 1e-10);

    // 接線ベクトルは単位ベクトル
    assert!((t0.length() - 1.0f64).abs() < 1e-10);
    assert!((t25.length() - 1.0f64).abs() < 1e-10);
}

/// 点の包含判定テスト
#[test]
fn test_point_containment() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 5.0).unwrap();

    // 中心点（内部）
    assert!(circle.contains_point_inside(&Point2D::new(2.0, 3.0)));

    // 円上の点
    let on_circle = Point2D::new(7.0, 3.0); // 右端
    assert!(circle.contains_point_on_circle(&on_circle, 1e-10));
    assert!(circle.contains_point_inside(&on_circle));

    // 外部の点
    let outside = Point2D::new(10.0, 3.0);
    assert!(!circle.contains_point_inside(&outside));
    assert!(!circle.contains_point_on_circle(&outside, 1e-10));
}

/// 距離計算テスト
#[test]
fn test_distance_calculations() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

    // 中心からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(0.0, 0.0)), 0.0);

    // 円上の点からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(3.0, 0.0)), 0.0);

    // 外部の点からの距離
    let outside_point = Point2D::new(6.0, 0.0);
    assert_eq!(circle.distance_to_point(&outside_point), 3.0);

    // 内部の点からの距離（0）
    assert_eq!(circle.distance_to_point(&Point2D::new(1.0, 0.0)), 0.0);
}

/// 変形操作テスト
#[test]
fn test_transformations() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // スケール
    let scaled = circle.scale(2.0).unwrap();
    assert_eq!(scaled.center(), Point2D::new(1.0, 2.0));
    assert_eq!(scaled.radius(), 6.0);

    // 無効なスケール
    assert!(circle.scale(-1.0).is_none());
    assert!(circle.scale(0.0).is_none());

    // 平行移動
    let offset = Vector2D::new(2.0, -1.0);
    let translated = circle.translate(offset);
    assert_eq!(translated.center(), Point2D::new(3.0, 1.0));
    assert_eq!(translated.radius(), 3.0);

    // 移動
    let new_center = Point2D::new(5.0, 5.0);
    let moved = circle.move_to(new_center);
    assert_eq!(moved.center(), new_center);
    assert_eq!(moved.radius(), 3.0);
}

/// 円同士の関係テスト
#[test]
fn test_circle_relationships() {
    let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();
    let circle2 = Circle2D::new(Point2D::new(4.0, 0.0), 2.0).unwrap(); // 交差
    let circle3 = Circle2D::new(Point2D::new(1.0, 0.0), 1.0).unwrap(); // 内包
    let circle4 = Circle2D::new(Point2D::new(10.0, 0.0), 1.0).unwrap(); // 離れている

    // 交差判定
    assert!(circle1.intersects_circle(&circle2)); // 交差する
    assert!(!circle1.intersects_circle(&circle3)); // 内包関係（交差しない）
    assert!(!circle1.intersects_circle(&circle4)); // 離れている

    // 包含判定
    assert!(circle1.contains_circle(&circle3)); // circle1がcircle3を包含
    assert!(!circle1.contains_circle(&circle2)); // 交差関係
    assert!(!circle1.contains_circle(&circle4)); // 離れている
}

/// 境界ボックステスト
#[test]
fn test_bounding_box() {
    let circle = Circle2D::new(Point2D::new(2.0, 3.0), 1.5).unwrap();
    let bbox = circle.bounding_box();

    assert_eq!(bbox.min(), Point2D::new(0.5, 1.5));
    assert_eq!(bbox.max(), Point2D::new(3.5, 4.5));
    assert_eq!(bbox.width(), 3.0);
    assert_eq!(bbox.height(), 3.0);
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let circle2d = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
    let circle3d = circle2d.to_3d();

    assert_eq!(circle3d.center().x(), 1.0);
    assert_eq!(circle3d.center().y(), 2.0);
    assert_eq!(circle3d.center().z(), 0.0);
    assert_eq!(circle3d.radius(), 3.0);

    // Z値指定での変換
    let circle3d_z = circle2d.to_3d_at_z(5.0);
    assert_eq!(circle3d_z.center().z(), 5.0);
}

/// Foundation trait - CoreFoundationテスト
#[test]
fn test_geometry_foundation() {
    let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

    // 境界ボックス取得
    let bbox = circle.bounding_box();
    assert_eq!(bbox.min(), Point2D::new(-2.0, -1.0));
    assert_eq!(bbox.max(), Point2D::new(4.0, 5.0));
}

/// Foundation trait - BasicMetricsテスト
#[test]
fn test_basic_metrics() {
    let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

    // 長さ（円周）
    use geo_foundation::core_foundation::BasicMetrics;
    let length = BasicMetrics::length(&circle).unwrap();
    assert!((length - TAU * 2.0).abs() < 1e-10);

    // 面積
    let area = BasicMetrics::area(&circle).unwrap();
    assert!((area - PI * 4.0).abs() < 1e-10);

    // 周長（円周と同じ）
    let perimeter = BasicMetrics::perimeter(&circle).unwrap();
    assert!((perimeter - TAU * 2.0).abs() < 1e-10);

    // 体積は定義されない
    assert!(BasicMetrics::volume(&circle).is_none());
}

/// Foundation trait - BasicContainmentテスト
#[test]
fn test_basic_containment() {
    let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

    let inside = Point2D::new(1.0, 1.0); // 中心
    let on_boundary = Point2D::new(3.0, 1.0); // 円上
    let outside = Point2D::new(5.0, 1.0); // 外部

    // 包含判定
    assert!(circle.contains_point(&inside));
    assert!(circle.contains_point(&on_boundary));
    assert!(!circle.contains_point(&outside));

    // 境界判定
    assert!(!circle.on_boundary(&inside, 1e-10));
    assert!(circle.on_boundary(&on_boundary, 1e-10));
    assert!(!circle.on_boundary(&outside, 1e-10));

    // 距離計算
    assert_eq!(circle.distance_to_point(&inside), 0.0);
    assert_eq!(circle.distance_to_point(&on_boundary), 0.0);
    assert_eq!(circle.distance_to_point(&outside), 2.0);
}

/// Foundation trait - BasicParametricテスト
#[test]
fn test_basic_parametric() {
    let circle = Circle2D::new(Point2D::new(0.0f64, 0.0f64), 1.0f64).unwrap();

    // パラメータ範囲 (旧仕様: 0-1正規化)
    let (start, end) = circle.parameter_range();
    assert_eq!(start, 0.0);
    assert_eq!(end, 1.0);

    // パラメータでの点取得 (0-1正規化、内部でTAU倍)
    let p0 = circle.point_at_parameter(0.0); // 0 * TAU = 0°
    let p25 = circle.point_at_parameter(0.25); // 0.25 * TAU = 90°
    let p50 = circle.point_at_parameter(0.5); // 0.5 * TAU = 180°

    assert!((p0.x() - 1.0f64).abs() < 1e-10);
    assert!((p25.y() - 1.0f64).abs() < 1e-10);
    assert!((p50.x() - (-1.0f64)).abs() < 1e-10);

    // 接線ベクトル取得
    let t0 = circle.tangent_at_parameter(0.0);
    let t25 = circle.tangent_at_parameter(0.25);

    assert!((t0.length() - 1.0f64).abs() < 1e-10);
    assert!((t25.length() - 1.0f64).abs() < 1e-10);
}

/// f32での動作テスト
#[test]
fn test_circle2d_f32() {
    let circle: Circle2D<f32> = Circle2D::new(Point2D::new(1.0f32, 2.0f32), 3.0f32).unwrap();

    assert_eq!(circle.radius(), 3.0f32);
    assert_eq!(circle.diameter(), 6.0f32);

    let area = circle.area();
    assert!((area - (std::f32::consts::PI * 9.0f32)).abs() < 1e-6);
}

// ============================================================================
// Foundation System Tests
// ============================================================================

#[cfg(test)]


    /// Foundation Extensions統合テスト
    #[test]
    fn test_foundation_extensions() {
        let circle = Circle2D::new(Point2D::new(2.0, 2.0), 3.0).unwrap();

        // Foundation scale from point
        let scaled = circle
            .foundation_scale_from_point(Point2D::new(0.0, 0.0), 2.0)
            .unwrap();
        assert_eq!(scaled.center(), Point2D::new(4.0, 4.0));
        assert_eq!(scaled.radius(), 6.0);

        // Foundation collision resolution
        let circle1 = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();
        let circle2 = Circle2D::new(Point2D::new(1.0, 0.0), 2.0).unwrap();
        let resolved = circle1.foundation_resolve_collision(&circle2);
        assert!(resolved.is_some());

        let (new_circle1, new_circle2) = resolved.unwrap();
        let new_distance = new_circle1.center().distance_to(&new_circle2.center());
        assert!((new_distance - 4.0_f64).abs() < 1e-10); // 半径の合計

        // Foundation weighted center
        let others = vec![Circle2D::new(Point2D::new(4.0, 0.0), 1.0).unwrap()];
        let weights = vec![1.0_f64];
        let weighted_center = circle.foundation_weighted_center(&others, &weights);
        assert!(weighted_center.is_some());
    }

    /// Foundation System数学的整合性テスト
    #[test]
    fn test_foundation_mathematical_consistency() {
        let circle = Circle2D::new(Point2D::new(1.0, 1.0), 2.0).unwrap();

        // スケール変換の数学的整合性
        let center_point = Point2D::new(0.0, 0.0);
        let factor = 1.5;
        let scaled = circle
            .foundation_scale_from_point(center_point, factor)
            .unwrap();

        // 期待値：center' = (0,0) + ((1,1) - (0,0)) * 1.5 = (1.5, 1.5)
        assert!((scaled.center().x() - 1.5_f64).abs() < 1e-10);
        assert!((scaled.center().y() - 1.5_f64).abs() < 1e-10);
        assert!((scaled.radius() - 3.0_f64).abs() < 1e-10);

        // Foundation transform の数学的一貫性
        let original_area = circle.area();
        let doubled = circle.foundation_transform("double_radius").unwrap();
        let expected_area = PI * (doubled.radius() * doubled.radius());
        assert!((doubled.area() - expected_area).abs() < 1e-10);
    }
}

// ============================================================================
// 新階層Foundation システムテスト
// ============================================================================

#[cfg(test)]
mod hierarchy_foundation_tests {
    use super::*;

    /// 新階層Foundation：マーカーInterface テスト
    #[test]
    fn test_marker_interfaces() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();

        // マーカーInterface の型チェック（コンパイル時確認）
        fn assert_geometry_shape<T: Scalar, S: GeometryShape<T>>(_: &S) {}
        fn assert_shape_2d<T: Scalar, S: Shape2D<T>>(_: &S) {}
        fn assert_surface_shape<T: Scalar, S: SurfaceShape<T>>(_: &S) {}
        fn assert_parametric_shape<T: Scalar, S: ParametricShape<T>>(_: &S) {}

        assert_geometry_shape(&circle);
        assert_shape_2d(&circle);
        assert_surface_shape(&circle);
        assert_parametric_shape(&circle);

        // 型情報の確認
        type Point = <Circle2D<f64> as GeometryShape<f64>>::Point;
        type Vector = <Circle2D<f64> as GeometryShape<f64>>::Vector;
        type BBox = <Circle2D<f64> as GeometryShape<f64>>::BBox;

        // 型の正確性確認
        let _: Point = Point2D::new(1.0, 2.0);
        let _: Vector = Vector2D::new(1.0, 2.0);
    }

    /// レベル1：データアクセス Foundation テスト
    #[test]
    fn test_data_access_level() {
        let circle = Circle2D::new(Point2D::new(2.0, 3.0), 4.0).unwrap();

        // DataAccess trait経由での境界ボックス取得
        let bbox = circle.bounding_box();

        // 境界ボックスの正確性確認
        assert_eq!(bbox.min().x(), -2.0); // 2.0 - 4.0
        assert_eq!(bbox.min().y(), -1.0); // 3.0 - 4.0
        assert_eq!(bbox.max().x(), 6.0); // 2.0 + 4.0
        assert_eq!(bbox.max().y(), 7.0); // 3.0 + 4.0
    }

    /// レベル2：基本計量 Foundation テスト
    #[test]
    fn test_basic_measurement_level() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0).unwrap();

        // BasicMeasurement trait経由での計量
        let area = BasicMeasurement::area(&circle).expect("Circle area should be Some");
        let perimeter =
            BasicMeasurement::perimeter(&circle).expect("Circle perimeter should be Some");

        // 数学的正確性確認
        assert!((area - (PI * 9.0)).abs() < 1e-10); // π * r²
        assert!((perimeter - (TAU * 3.0)).abs() < 1e-10); // 2π * r

        // length は円では定義されない
        assert!(circle.length().is_none());
    }

    /// レベル3：基本包含 Foundation テスト
    #[test]
    fn test_basic_containment_level() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 5.0).unwrap();

        // BasicContainment trait経由での包含判定
        assert!(circle.contains_point(&Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(&Point2D::new(3.0, 4.0))); // 内部点
        assert!(!circle.contains_point(&Point2D::new(6.0, 0.0))); // 外部点

        // 距離計算
        assert_eq!(circle.distance_to_point(&Point2D::new(0.0, 0.0)), 0.0); // 内部
        assert_eq!(circle.distance_to_point(&Point2D::new(10.0, 0.0)), 5.0); // 外部

        // 境界判定
        assert!(circle.on_boundary(&Point2D::new(5.0, 0.0), 0.001)); // 境界上
        assert!(!circle.on_boundary(&Point2D::new(10.0, 0.0), 0.001)); // 境界外
    }

    /// レベル4：パラメトリック Foundation テスト
    #[test]
    fn test_basic_parametric_level() {
        let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();

        // NewBasicParametric trait経由でのパラメトリック操作
        let point_0 = NewBasicParametric::point_at_parameter(&circle, 0.0);
        let point_pi_2 = NewBasicParametric::point_at_parameter(&circle, PI / 2.0);
        let point_pi = NewBasicParametric::point_at_parameter(&circle, PI);

        // パラメトリック点の正確性
        assert!((point_0.x() - 4.0).abs() < 1e-10); // 1 + 3*cos(0) = 4
        assert!((point_0.y() - 2.0).abs() < 1e-10); // 2 + 3*sin(0) = 2

        assert!((point_pi_2.x() - 1.0).abs() < 1e-10); // 1 + 3*cos(π/2) = 1
        assert!((point_pi_2.y() - 5.0).abs() < 1e-10); // 2 + 3*sin(π/2) = 5

        assert!((point_pi.x() - (-2.0)).abs() < 1e-10); // 1 + 3*cos(π) = -2
        assert!((point_pi.y() - 2.0).abs() < 1e-10); // 2 + 3*sin(π) = 2

        // 接線ベクトル
        let tangent_0 = NewBasicParametric::tangent_at_parameter(&circle, 0.0);
        assert!((tangent_0.x() - 0.0).abs() < 1e-10); // -3*sin(0) = 0
        assert!((tangent_0.y() - 3.0).abs() < 1e-10); // 3*cos(0) = 3

        // パラメータ範囲
        let (min_t, max_t) = NewBasicParametric::parameter_range(&circle);
        assert_eq!(min_t, 0.0);
        assert_eq!(max_t, TAU);
    }

    /// 円特化Foundation テスト
    #[test]
    fn test_circular_foundation() {
        let circle = Circle2D::new(Point2D::new(3.0, 4.0), 2.5).unwrap();

        // CircularFoundation trait経由での円特化操作
        let center = circle.center();
        let radius = circle.radius();

        assert_eq!(center, Point2D::new(3.0, 4.0));
        assert_eq!(radius, 2.5);
    }

    /// 階層Foundation統合テスト
    #[test]
    fn test_foundation_hierarchy_integration() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        // 全レベルの機能を段階的に確認
        // レベル1: データアクセス
        let bbox = circle.bounding_box();
        let test_point = Point2D::new(0.5, 0.5);
        assert!(
            test_point.x() >= bbox.min().x()
                && test_point.x() <= bbox.max().x()
                && test_point.y() >= bbox.min().y()
                && test_point.y() <= bbox.max().y()
        );

        // レベル2: 計量 (レベル1継承)
        let area = BasicMeasurement::area(&circle).expect("Circle area should be Some");
        assert!((area - PI).abs() < 1e-10);

        // レベル3: 包含 (レベル2継承)
        assert!(circle.contains_point(&Point2D::new(0.5, 0.0)));

        // レベル4: パラメトリック (レベル3継承)
        let point = circle.point_at_parameter(PI / 4.0);
        assert!(circle.contains_point(&point));

        // 特化: 円固有 (全レベル利用可能)
        assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
        assert_eq!(circle.radius(), 1.0);
    }
}
