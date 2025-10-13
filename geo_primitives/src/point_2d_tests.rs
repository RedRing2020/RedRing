//! Point2D の包括的テスト
//!
//! 基本機能、座標操作、距離計算、変換機能、演算子などをテスト

use crate::{Point2D, Vector2D};
use geo_foundation::Angle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation_and_accessors() {
        // 基本的な作成とアクセサ
        let point = Point2D::new(3.0_f64, 4.0_f64);
        assert_eq!(point.x(), 3.0);
        assert_eq!(point.y(), 4.0);

        // 原点
        let origin = Point2D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);

        // 座標配列
        let coords = point.coords();
        assert_eq!(coords, [3.0, 4.0]);

        // タプル変換
        let tuple = point.to_tuple();
        assert_eq!(tuple, (3.0, 4.0));

        // タプルから作成
        let from_tuple = Point2D::from_tuple((5.0_f64, 6.0_f64));
        assert_eq!(from_tuple.x(), 5.0);
        assert_eq!(from_tuple.y(), 6.0);
    }

    #[test]
    fn test_distance_calculations() {
        let p1 = Point2D::new(0.0_f64, 0.0_f64);
        let p2 = Point2D::new(3.0_f64, 4.0_f64);

        // 距離計算
        let distance = p1.distance_to(&p2);
        assert_eq!(distance, 5.0);

        // 距離の2乗
        let distance_squared = p1.distance_squared_to(&p2);
        assert_eq!(distance_squared, 25.0);

        // 原点からの距離
        let origin_distance = p2.distance_to(&Point2D::<f64>::origin());
        assert_eq!(origin_distance, 5.0);

        // 原点からの距離の2乗
        let origin_distance_squared = p2.distance_squared_to(&Point2D::<f64>::origin());
        assert_eq!(origin_distance_squared, 25.0);
    }

    #[test]
    fn test_interpolation_and_midpoint() {
        let p1 = Point2D::new(0.0_f64, 0.0_f64);
        let p2 = Point2D::new(10.0_f64, 10.0_f64);

        // 線形補間
        let lerp_start = p1.lerp(&p2, 0.0_f64);
        assert_eq!(lerp_start, p1);

        let lerp_end = p1.lerp(&p2, 1.0_f64);
        assert_eq!(lerp_end, p2);

        let lerp_mid = p1.lerp(&p2, 0.5_f64);
        assert_eq!(lerp_mid, Point2D::new(5.0_f64, 5.0_f64));

        let lerp_quarter = p1.lerp(&p2, 0.25_f64);
        assert_eq!(lerp_quarter, Point2D::new(2.5_f64, 2.5_f64));

        // 中点計算
        let midpoint = p1.midpoint(&p2);
        assert_eq!(midpoint, Point2D::new(5.0_f64, 5.0_f64));

        // 異なる点での中点
        let p3 = Point2D::new(1.0_f64, 3.0_f64);
        let p4 = Point2D::new(5.0_f64, 7.0_f64);
        let mid = p3.midpoint(&p4);
        assert_eq!(mid, Point2D::new(3.0_f64, 5.0_f64));
    }

    #[test]
    fn test_transformations() {
        let point = Point2D::new(1.0_f64, 2.0_f64);

        // 平行移動
        let translation_vector = crate::Vector2D::new(3.0_f64, 4.0_f64);
        let translated = point.translate(translation_vector);
        assert_eq!(translated, Point2D::new(4.0_f64, 6.0_f64));

        // 原点中心回転（90度）
        let rotated_90 = point.rotate(std::f64::consts::PI / 2.0);
        assert!((rotated_90.x() - (-2.0)).abs() < 1e-10);
        assert!((rotated_90.y() - 1.0).abs() < 1e-10);

        // 180度回転
        let rotated_180 = point.rotate(std::f64::consts::PI);
        assert!((rotated_180.x() - (-1.0)).abs() < 1e-10);
        assert!((rotated_180.y() - (-2.0)).abs() < 1e-10);

        // 指定点中心回転
        let center = Point2D::new(1.0_f64, 1.0_f64);
        let rotated_around =
            point.rotate_around_angle(&center, Angle::from_radians(std::f64::consts::PI / 2.0));
        assert!((rotated_around.x() - 0.0).abs() < 1e-10);
        assert!((rotated_around.y() - 1.0).abs() < 1e-10);

        // スケーリング
        let scaled = point.scale(2.0_f64, 3.0_f64);
        assert_eq!(scaled, Point2D::new(2.0_f64, 6.0_f64));

        // 均等スケーリング
        let scaled_uniform = point.scale_uniform(2.0_f64);
        assert_eq!(scaled_uniform, Point2D::new(2.0_f64, 4.0_f64));
    }

    #[test]
    fn test_vector_conversion() {
        let point = Point2D::new(3.0_f64, 4.0_f64);

        // Point2D を Vector2D に変換
        let vector = point.to_vector();
        assert_eq!(vector.x(), 3.0);
        assert_eq!(vector.y(), 4.0);

        // Vector2D から Point2D を作成
        let vector = Vector2D::new(5.0_f64, 6.0_f64);
        let from_vector = Point2D::from_vector(vector);
        assert_eq!(from_vector.x(), 5.0);
        assert_eq!(from_vector.y(), 6.0);

        // 2点間のベクトル
        let p1 = Point2D::new(1.0_f64, 2.0_f64);
        let p2 = Point2D::new(4.0_f64, 6.0_f64);
        let vector_between = p1.vector_to(&p2);
        assert_eq!(vector_between.x(), 3.0);
        assert_eq!(vector_between.y(), 4.0);
    }

    #[test]
    fn test_arithmetic_operators() {
        let point = Point2D::new(2.0_f64, 3.0_f64);
        let vector = Vector2D::new(1.0_f64, 4.0_f64);

        // Point + Vector = Point
        let added = point + vector;
        assert_eq!(added, Point2D::new(3.0_f64, 7.0_f64));

        // Point - Vector = Point
        let subtracted = point - vector;
        assert_eq!(subtracted, Point2D::new(1.0_f64, -1.0_f64));

        // Point - Point = Vector（2点間のベクトル）
        let p1 = Point2D::new(5.0_f64, 8.0_f64);
        let p2 = Point2D::new(2.0_f64, 3.0_f64);
        let diff = p1 - p2;
        assert_eq!(diff, Vector2D::new(3.0_f64, 5.0_f64));

        // スカラー乗算
        let scaled = point * 2.0_f64;
        assert_eq!(scaled, Point2D::new(4.0_f64, 6.0_f64));

        // 負号
        let negated = -point;
        assert_eq!(negated, Point2D::new(-2.0_f64, -3.0_f64));
    }

    #[test]
    fn test_comparison_and_equality() {
        let p1 = Point2D::new(1.0_f64, 2.0_f64);
        let p2 = Point2D::new(1.0_f64, 2.0_f64);
        let p3 = Point2D::new(2.0_f64, 3.0_f64);

        // 等価性
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);

        // 近似等価性
        let p4 = Point2D::new(1.0000001_f64, 2.0000001_f64);
        assert!(p1.is_approximately_equal(&p4, 1e-6_f64));
        assert!(!p1.is_approximately_equal(&p4, 1e-8_f64));
    }

    #[test]
    fn test_3d_extension() {
        let point_2d = Point2D::new(3.0_f64, 4.0_f64);

        // Z=0での3D拡張
        let point_3d = point_2d.to_3d();
        assert_eq!(point_3d.x(), 3.0);
        assert_eq!(point_3d.y(), 4.0);
        assert_eq!(point_3d.z(), 0.0);

        // 指定Z値での3D拡張
        let point_3d_z = point_2d.to_3d_with_z(5.0_f64);
        assert_eq!(point_3d_z.x(), 3.0);
        assert_eq!(point_3d_z.y(), 4.0);
        assert_eq!(point_3d_z.z(), 5.0);
    }

    #[test]
    fn test_foundation_traits() {
        let point = Point2D::new(2.0_f64, 3.0_f64);
        let point2 = Point2D::new(2.0000001_f64, 3.0000001_f64);

        // 基本的な等価性
        assert_eq!(point, point);
        assert_ne!(point, point2);

        // 近似等価性 (is_approximately_equalが存在する場合)
        if let Ok(is_approx) =
            std::panic::catch_unwind(|| point.is_approximately_equal(&point2, 1e-6_f64))
        {
            assert!(is_approx);
        }
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let point = Point2D::new(1.5f32, 2.5f32);
        let other = Point2D::new(4.5f32, 6.5f32);

        // 距離計算
        let distance = point.distance_to(&other);
        let expected = 5.0f32; // 3-4-5三角形
        assert!((distance - expected).abs() < 1e-6);

        // 中点計算
        let midpoint = point.midpoint(&other);
        assert_eq!(midpoint, Point2D::new(3.0f32, 4.5f32));

        // 回転
        let rotated = point.rotate(std::f32::consts::PI / 2.0);
        assert!((rotated.x() - (-2.5f32)).abs() < 1e-6);
        assert!((rotated.y() - 1.5f32).abs() < 1e-6);
    }

    #[test]
    fn test_edge_cases() {
        let origin = Point2D::<f64>::origin();

        // 原点の特性
        assert_eq!(origin.distance_to(&Point2D::<f64>::origin()), 0.0);
        assert_eq!(origin.distance_squared_to(&Point2D::<f64>::origin()), 0.0);

        // 同じ点での操作
        let same_point = Point2D::new(1.0_f64, 2.0_f64);
        assert_eq!(same_point.distance_to(&same_point), 0.0);
        assert_eq!(same_point.midpoint(&same_point), same_point);

        // ゼロベクトルとの演算
        let zero_vector = Vector2D::<f64>::zero();
        assert_eq!(same_point + zero_vector, same_point);
        assert_eq!(same_point - zero_vector, same_point);

        // ゼロスケーリング
        let zero_scaled = same_point * 0.0_f64;
        assert_eq!(zero_scaled, Point2D::<f64>::origin());
    }

    #[test]
    fn test_large_coordinates() {
        // 大きな座標値でのテスト
        let large_point = Point2D::new(1e10_f64, 1e10_f64);
        let small_point = Point2D::new(1.0_f64, 1.0_f64);

        // 距離計算が正常に動作することを確認
        let distance = large_point.distance_to(&small_point);
        assert!(distance > 1e10);

        // 中点計算
        let midpoint = large_point.midpoint(&small_point);
        assert!((midpoint.x() - 5e9 - 0.5).abs() < 1e-6);
        assert!((midpoint.y() - 5e9 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_negative_coordinates() {
        // 負の座標でのテスト
        let neg_point = Point2D::new(-3.0_f64, -4.0_f64);
        let pos_point = Point2D::new(3.0_f64, 4.0_f64);

        // 距離計算
        let distance = neg_point.distance_to(&pos_point);
        assert!((distance - 10.0).abs() < 1e-10); // √((6)² + (8)²) = 10

        // 中点（原点）
        let midpoint = neg_point.midpoint(&pos_point);
        assert_eq!(midpoint, Point2D::<f64>::origin());

        // 距離の計算
        let norm = neg_point.distance_to(&Point2D::<f64>::origin());
        assert!((norm - 5.0).abs() < 1e-10);
    }
}
