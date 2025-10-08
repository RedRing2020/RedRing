//! Ray型の基礎動作テスト
//!
//! Ray2D<T>, Ray3D<T>の基本操作（起点、方向、点の計算など）の動作確認

#[cfg(test)]
mod tests {
    use crate::geometry2d::{Point2D, Ray2D, Vector as Vector2D};
    use crate::geometry3d::{Point3D, Ray3D, Vector as Vector3D};
    use geo_foundation::abstract_types::geometry::Ray;
    use geo_foundation::abstract_types::{Angle, Scalar};

    #[test]
    fn test_ray2d_basic_operations_f64() {
        let origin = Point2D::<f64>::new(1.0, 2.0);
        let direction_vec = Vector2D::new(1.0, 0.0);

        let ray = Ray2D::from_origin_and_vector(origin, direction_vec).unwrap();

        // 基本プロパティ
        assert_eq!(ray.origin().x(), 1.0);
        assert_eq!(ray.origin().y(), 2.0);
        assert_eq!(ray.direction().x(), 1.0);
        assert_eq!(ray.direction().y(), 0.0);

        // パラメータ指定点の計算
        let point_at_2 = ray.point_at_parameter(2.0).unwrap();
        assert_eq!(point_at_2.x(), 3.0);
        assert_eq!(point_at_2.y(), 2.0);

        // 負のパラメータは無効（半無限直線）
        assert!(ray.point_at_parameter(-1.0).is_none());

        // 点が含まれるかの判定
        let test_point = Point2D::new(3.0, 2.0);
        assert!(ray.contains_point(&test_point, f64::TOLERANCE));

        // レイ上にない点
        let off_ray_point = Point2D::new(3.0, 3.0);
        assert!(!ray.contains_point(&off_ray_point, f64::TOLERANCE));
    }

    #[test]
    fn test_ray2d_distance_and_closest_point() {
        let origin = Point2D::<f64>::origin();
        let ray = Ray2D::x_axis(origin); // X軸正方向のレイ

        // レイ上の点への距離（ゼロ）
        let on_ray_point = Point2D::new(5.0, 0.0);
        assert!((ray.distance_to_point(&on_ray_point) - 0.0).abs() < f64::TOLERANCE);

        // レイ上の最近点
        let closest = ray.closest_point(&on_ray_point);
        assert_eq!(closest.x(), 5.0);
        assert_eq!(closest.y(), 0.0);

        // レイから離れた点
        let off_point = Point2D::new(3.0, 4.0);
        let distance = ray.distance_to_point(&off_point);
        assert!((distance - 4.0).abs() < f64::TOLERANCE); // Y方向に4離れている

        let closest_to_off = ray.closest_point(&off_point);
        assert_eq!(closest_to_off.x(), 3.0);
        assert_eq!(closest_to_off.y(), 0.0);

        // 起点より後方の点
        let behind_point = Point2D::new(-2.0, 3.0);
        let closest_behind = ray.closest_point(&behind_point);
        assert_eq!(closest_behind.x(), 0.0); // 起点が最近点
        assert_eq!(closest_behind.y(), 0.0);
    }

    #[test]
    fn test_ray2d_parameter_calculation() {
        let origin = Point2D::<f64>::new(1.0, 1.0);
        let direction_vec = Vector2D::new(3.0, 4.0); // 長さ5のベクトル（正規化される）
        let ray = Ray2D::from_origin_and_vector(origin, direction_vec).unwrap();

        // 起点でのパラメータ
        let param_at_origin = ray.parameter_at_point(&origin);
        assert!((param_at_origin - 0.0).abs() < f64::TOLERANCE);

        // 方向に5進んだ点（正規化されているので実際には長さ1のベクトル）
        let forward_point = Point2D::new(1.0 + 3.0 / 5.0, 1.0 + 4.0 / 5.0);
        let param_forward = ray.parameter_at_point(&forward_point);
        assert!((param_forward - 1.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_ray2d_parallel_and_coincident() {
        let ray1 = Ray2D::x_axis(Point2D::<f64>::origin());
        let ray2 = Ray2D::x_axis(Point2D::new(0.0, 1.0)); // 平行だが異なる位置
        let ray3 = Ray2D::x_axis(Point2D::new(2.0, 0.0)); // 同一直線上

        // 平行性テスト
        assert!(ray1.is_parallel_to(&ray2, f64::TOLERANCE));
        assert!(ray1.is_parallel_to(&ray3, f64::TOLERANCE));

        // 同一性テスト
        assert!(!ray1.is_coincident_with(&ray2, f64::TOLERANCE)); // 平行だが別の直線
        assert!(ray1.is_coincident_with(&ray3, f64::TOLERANCE)); // 同一直線上
    }

    #[test]
    fn test_ray2d_transformations() {
        let origin = Point2D::<f64>::new(1.0, 1.0);
        let ray = Ray2D::x_axis(origin);

        // 平行移動
        let translation = Vector2D::new(2.0, 3.0);
        let translated = ray.translate(&translation);
        assert_eq!(translated.origin().x(), 3.0);
        assert_eq!(translated.origin().y(), 4.0);
        assert_eq!(translated.direction().x(), 1.0); // 方向は変わらない
        assert_eq!(translated.direction().y(), 0.0);

        // 90度回転
        let ninety_degrees = Angle::from_degrees(90.0);
        let rotated = ray.rotate(ninety_degrees);
        // 起点も回転する
        assert!((rotated.direction().x() - 0.0).abs() < f64::TOLERANCE);
        assert!((rotated.direction().y() - 1.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_ray3d_basic_operations_f64() {
        let origin = Point3D::<f64>::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);

        let ray = Ray3D::new(origin, direction).unwrap();

        // 基本プロパティ
        assert_eq!(ray.origin().x(), 1.0);
        assert_eq!(ray.origin().y(), 2.0);
        assert_eq!(ray.origin().z(), 3.0);

        // パラメータ指定点の計算
        let point_at_2 = ray.point_at_parameter(2.0).unwrap();
        assert_eq!(point_at_2.x(), 3.0);
        assert_eq!(point_at_2.y(), 2.0);
        assert_eq!(point_at_2.z(), 3.0);

        // 軸レイの作成
        let x_ray = Ray3D::<f64>::x_axis(Point3D::origin());
        let y_ray = Ray3D::<f64>::y_axis(Point3D::origin());
        let z_ray = Ray3D::<f64>::z_axis(Point3D::origin());

        assert_eq!(x_ray.direction_vector().x(), 1.0);
        assert_eq!(y_ray.direction_vector().y(), 1.0);
        assert_eq!(z_ray.direction_vector().z(), 1.0);
    }

    #[test]
    fn test_ray3d_distance_operations() {
        let origin = Point3D::<f64>::origin();
        let ray = Ray3D::<f64>::x_axis(origin);

        // レイ上の点
        let on_ray = Point3D::new(5.0, 0.0, 0.0);
        assert!((ray.distance_to_point(&on_ray) - 0.0).abs() < f64::TOLERANCE);

        // レイから離れた点
        let off_ray = Point3D::new(3.0, 4.0, 0.0);
        let distance = ray.distance_to_point(&off_ray);
        assert!((distance - 4.0).abs() < f64::TOLERANCE);

        let closest = ray.closest_point(&off_ray);
        assert_eq!(closest.x(), 3.0);
        assert_eq!(closest.y(), 0.0);
        assert_eq!(closest.z(), 0.0);
    }

    #[test]
    fn test_ray3d_parallel_operations() {
        let ray1 = Ray3D::<f64>::x_axis(Point3D::origin());
        let ray2 = Ray3D::<f64>::x_axis(Point3D::new(0.0, 1.0, 0.0));
        let ray3 = Ray3D::<f64>::y_axis(Point3D::origin());

        // 平行レイ
        assert!(ray1.is_parallel_to(&ray2, f64::TOLERANCE));

        // 非平行レイ
        assert!(!ray1.is_parallel_to(&ray3, f64::TOLERANCE));
    }

    #[test]
    fn test_ray3d_rotations() {
        let origin = Point3D::<f64>::new(1.0, 1.0, 1.0);
        let ray = Ray3D::<f64>::x_axis(origin);

        // X軸周り90度回転
        let ninety_degrees = Angle::from_degrees(90.0);
        let rotated_x = ray.rotate_x(ninety_degrees);

        // 方向ベクトルはX軸方向のまま（X軸周り回転なので）
        assert!((rotated_x.direction_vector().x() - 1.0).abs() < f64::TOLERANCE);
        assert!((rotated_x.direction_vector().y() - 0.0).abs() < f64::TOLERANCE);
        assert!((rotated_x.direction_vector().z() - 0.0).abs() < f64::TOLERANCE);

        // Y軸周り90度回転
        let rotated_y = ray.rotate_y(ninety_degrees);
        // X軸方向が-Z軸方向になる（右手座標系でY軸周り90度回転）
        assert!((rotated_y.direction_vector().x() - 0.0).abs() < f64::TOLERANCE);
        assert!((rotated_y.direction_vector().y() - 0.0).abs() < f64::TOLERANCE);
        assert!((rotated_y.direction_vector().z() - (-1.0)).abs() < f64::TOLERANCE);

        // オイラー角回転テスト
        let euler_rotated = ray.rotate_euler(
            Angle::from_degrees(30.0),
            Angle::from_degrees(45.0),
            Angle::from_degrees(60.0),
        );
        // 複数回転後も正規化された方向ベクトルのまま
        let length = euler_rotated.direction_vector().length();
        assert!((length - 1.0).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_ray_type_compatibility() {
        // f32とf64両方でコンパイルできることを確認
        let _ray2d_f32 = Ray2D::<f32>::x_axis(Point2D::origin());
        let _ray2d_f64 = Ray2D::<f64>::x_axis(Point2D::origin());

        let _ray3d_f32 = Ray3D::<f32>::x_axis(Point3D::origin());
        let _ray3d_f64 = Ray3D::<f64>::x_axis(Point3D::origin());
    }
}
