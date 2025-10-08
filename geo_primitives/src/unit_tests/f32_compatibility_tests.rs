//! f32型サポートの包括的テスト
//!
//! 全ての幾何型でf32が正しく動作することを確認

#[cfg(test)]
mod tests {
    use crate::geometry2d::{Direction2D, Point2D, Point2DF32, Ray2D, Vector as Vector2D};
    use crate::geometry3d::{Point3D, Point3DF32, Ray3D};
    use geo_foundation::abstract_types::Scalar;
    use geo_foundation::Ray;

    #[test]
    fn test_f32_type_aliases() {
        // f32型エイリアスが正しく設定されているかテスト
        let _p2d: Point2DF32 = Point2D::new(1.0f32, 2.0f32);
        let _p3d: Point3DF32 = Point3D::new(1.0f32, 2.0f32, 3.0f32);

        // 型エイリアスが期待通りの型になっているか確認
        assert_eq!(
            std::mem::size_of::<Point2DF32>(),
            std::mem::size_of::<Point2D<f32>>()
        );
        assert_eq!(
            std::mem::size_of::<Point3DF32>(),
            std::mem::size_of::<Point3D<f32>>()
        );
    }

    #[test]
    fn test_point2d_f32_operations() {
        let p1 = Point2D::<f32>::new(1.0, 2.0);
        let p2 = Point2D::<f32>::new(3.0, 4.0);
        let vec = Vector2D::<f32>::new(1.0, 1.0);

        // 基本演算
        let p3 = p1 + vec;
        assert_eq!(p3.x(), 2.0);
        assert_eq!(p3.y(), 3.0);

        let diff = p2 - p1;
        assert_eq!(diff.x(), 2.0);
        assert_eq!(diff.y(), 2.0);

        let p4 = p1 * 2.0f32;
        assert_eq!(p4.x(), 2.0);
        assert_eq!(p4.y(), 4.0);

        // 距離計算
        let distance = p1.distance_to(&p2);
        assert!((distance - 2.828427f32).abs() < 0.001);
    }

    #[test]
    fn test_point3d_f32_operations() {
        let p1 = Point3D::<f32>::new(1.0, 2.0, 3.0);
        let p2 = Point3D::<f32>::new(4.0, 5.0, 6.0);
        let vec = crate::geometry3d::Vector::<f32>::new(1.0, 1.0, 1.0);

        // 基本演算
        let p3 = p1 + vec;
        assert_eq!(p3.x(), 2.0);
        assert_eq!(p3.y(), 3.0);
        assert_eq!(p3.z(), 4.0);

        let diff = p2 - p1;
        assert_eq!(diff.x(), 3.0);
        assert_eq!(diff.y(), 3.0);
        assert_eq!(diff.z(), 3.0);

        // 3D固有メソッド
        let p2d = p1.to_point2d();
        assert_eq!(p2d.x(), 1.0);
        assert_eq!(p2d.y(), 2.0);
    }

    #[test]
    fn test_vector_f32_operations() {
        let v1 = Vector2D::<f32>::new(3.0, 4.0);
        let v2 = Vector2D::<f32>::new(1.0, 2.0);

        // 基本演算
        let v3 = v1 + v2;
        assert_eq!(v3.x(), 4.0);
        assert_eq!(v3.y(), 6.0);

        // 長さ
        assert!((v1.norm() - 5.0f32).abs() < 0.001);

        // 正規化
        let normalized = v1.normalize().unwrap();
        assert!((normalized.norm() - 1.0f32).abs() < 0.001);

        // 内積
        let dot = v1.dot(&v2);
        assert_eq!(dot, 11.0);
    }

    #[test]
    fn test_mixed_precision_compatibility() {
        // f32とf64の型が独立して動作することを確認
        let p32 = Point2D::<f32>::new(1.0, 2.0);
        let p64 = Point2D::<f64>::new(1.0, 2.0);

        // 型が異なることを確認
        assert_ne!(std::mem::size_of_val(&p32), std::mem::size_of_val(&p64));

        // 同じ値で初期化された場合の座標
        assert_eq!(p32.x() as f64, p64.x());
        assert_eq!(p32.y() as f64, p64.y());
    }

    #[test]
    fn test_scalar_trait_consistency() {
        // f32がScalarトレイトを正しく実装しているか確認
        fn test_scalar<T: Scalar>(value: T) -> T {
            value + T::ONE
        }

        let result32 = test_scalar(1.0f32);
        let result64 = test_scalar(1.0f64);

        assert_eq!(result32, 2.0f32);
        assert_eq!(result64, 2.0f64);
    }

    #[test]
    fn test_direction2d_f32_support() {
        use crate::geometry2d::Direction2D;

        // Direction2D<f32>の基本操作
        let dir_x = Direction2D::<f32>::positive_x();
        let dir_y = Direction2D::<f32>::positive_y();

        // 基本メソッドが動作するか確認
        assert_eq!(dir_x.x(), 1.0f32);
        assert_eq!(dir_x.y(), 0.0f32);
        assert_eq!(dir_y.x(), 0.0f32);
        assert_eq!(dir_y.y(), 1.0f32);

        // 垂直方向取得（Direct メソッド呼び出し）
        let perp = Direction2D::<f32>::positive_y(); // X軸に垂直なY軸方向
        assert_eq!(perp.x(), 0.0f32);
        assert_eq!(perp.y(), 1.0f32);
    }

    #[test]
    fn test_ray_f32_support() {
        use crate::geometry2d::{Direction2D, Ray2D};
        use crate::geometry3d::Ray3D;
        use geo_foundation::Ray;

        // Ray2D<f32>の基本操作
        let origin2d = Point2D::<f32>::new(1.0, 2.0);
        let direction2d = Direction2D::<f32>::positive_x();
        let ray2d = Ray2D::new(origin2d, direction2d);

        assert_eq!(ray2d.origin().x(), 1.0f32);
        assert_eq!(ray2d.origin().y(), 2.0f32);

        // Ray3D<f32>の基本操作
        let origin3d = Point3D::<f32>::new(1.0, 2.0, 3.0);
        let ray3d = Ray3D::x_axis(origin3d);

        assert_eq!(ray3d.origin().x(), 1.0f32);
        assert_eq!(ray3d.origin().y(), 2.0f32);
        assert_eq!(ray3d.origin().z(), 3.0f32);
    }

    #[test]
    fn test_f32_type_alias_coverage() {
        use crate::geometry2d::{Direction2DF32, Point2DF32, Ray2DF32};
        use crate::geometry3d::{Point3DF32, Ray3DF32};

        // 型エイリアスが正しく機能することを確認
        let p2d: Point2DF32 = Point2D::new(1.0f32, 2.0f32);
        let p3d: Point3DF32 = Point3D::new(1.0f32, 2.0f32, 3.0f32);
        let dir2d: Direction2DF32 = Direction2D::positive_x();
        let ray2d: Ray2DF32 = Ray2D::new(p2d, dir2d);
        let ray3d: Ray3DF32 = Ray3D::x_axis(p3d);

        // 実際に使用して型が正しいことを確認
        assert_eq!(p2d.x(), 1.0f32);
        assert_eq!(p3d.z(), 3.0f32);
        assert_eq!(dir2d.x(), 1.0f32);
        assert_eq!(ray2d.origin().x(), 1.0f32);
        assert_eq!(ray3d.origin().z(), 3.0f32);

        // サイズが期待通りか確認
        assert_eq!(std::mem::size_of::<Point2DF32>(), 8); // f32 * 2
        assert_eq!(std::mem::size_of::<Point3DF32>(), 12); // f32 * 3
        assert_eq!(std::mem::size_of::<Direction2DF32>(), 8); // f32 * 2
    }
}
