//! Point型の基礎演算テスト
//! 
//! Point2D<T>, Point3D<T>の基礎演算（加算、減算、スカラー倍など）の動作確認

#[cfg(test)]
mod tests {
    use crate::geometry2d::{Point2D, Vector as Vector2D};
    use crate::geometry3d::{Point3D, Vector as Vector3D};
    use geo_foundation::abstract_types::Scalar;

    #[test]
    fn test_point2d_basic_operations_f64() {
        let p1 = Point2D::<f64>::new(1.0, 2.0);
        let p2 = Point2D::<f64>::new(3.0, 4.0);
        let vec = Vector2D::new(1.0, 1.0);

        // 基本アクセサ
        assert_eq!(p1.x(), 1.0);
        assert_eq!(p1.y(), 2.0);

        // 平行移動（Vectorとの加算）
        let p3 = p1 + vec;
        assert_eq!(p3.x(), 2.0);
        assert_eq!(p3.y(), 3.0);

        // Point間の減算（Vectorを返す）
        let diff = p2 - p1;
        assert_eq!(diff.x(), 2.0);
        assert_eq!(diff.y(), 2.0);

        // スカラー倍
        let p4 = p1 * 2.0;
        assert_eq!(p4.x(), 2.0);
        assert_eq!(p4.y(), 4.0);

        // スカラー除算
        let p5 = p4 / 2.0;
        assert_eq!(p5.x(), 1.0);
        assert_eq!(p5.y(), 2.0);

        // 距離計算
        let distance = p1.distance_to(&p2);
        assert!((distance - (8.0_f64).sqrt()).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_point2d_basic_operations_f32() {
        let p1 = Point2D::<f32>::new(1.0, 2.0);
        let vec = Vector2D::new(1.0, 1.0);

        // 平行移動
        let p2 = p1 + vec;
        assert_eq!(p2.x(), 2.0);
        assert_eq!(p2.y(), 3.0);

        // 回転（90度）
        let rotated = p1.rotate(std::f32::consts::PI / 2.0);
        assert!((rotated.x() - (-2.0)).abs() < f32::TOLERANCE);
        assert!((rotated.y() - 1.0).abs() < f32::TOLERANCE);
    }

    #[test]
    fn test_point2d_special_methods() {
        let p1 = Point2D::<f64>::new(0.0, 0.0);
        let p2 = Point2D::<f64>::new(4.0, 0.0);

        // 中点計算
        let midpoint = p1.midpoint(&p2);
        assert_eq!(midpoint.x(), 2.0);
        assert_eq!(midpoint.y(), 0.0);

        // 原点
        let origin = Point2D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
    }

    #[test]
    fn test_point3d_basic_operations_f64() {
        let p1 = Point3D::<f64>::new(1.0, 2.0, 3.0);
        let p2 = Point3D::<f64>::new(4.0, 5.0, 6.0);
        let vec = Vector3D::new(1.0, 1.0, 1.0);

        // 基本アクセサ
        assert_eq!(p1.x(), 1.0);
        assert_eq!(p1.y(), 2.0);
        assert_eq!(p1.z(), 3.0);

        // 平行移動（Vectorとの加算）
        let p3 = p1 + vec;
        assert_eq!(p3.x(), 2.0);
        assert_eq!(p3.y(), 3.0);
        assert_eq!(p3.z(), 4.0);

        // Point間の減算（Vectorを返す）
        let diff = p2 - p1;
        assert_eq!(diff.x(), 3.0);
        assert_eq!(diff.y(), 3.0);
        assert_eq!(diff.z(), 3.0);

        // スカラー倍
        let p4 = p1 * 2.0;
        assert_eq!(p4.x(), 2.0);
        assert_eq!(p4.y(), 4.0);
        assert_eq!(p4.z(), 6.0);

        // 距離計算
        let distance = p1.distance_to(&p2);
        assert!((distance - (27.0_f64).sqrt()).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_point3d_special_methods() {
        let p1 = Point3D::<f64>::new(1.0, 2.0, 3.0);
        let p2 = Point3D::<f64>::new(5.0, 6.0, 7.0);

        // 中点計算
        let midpoint = p1.midpoint(&p2);
        assert_eq!(midpoint.x(), 3.0);
        assert_eq!(midpoint.y(), 4.0);
        assert_eq!(midpoint.z(), 5.0);

        // 2D投影
        let projected = p1.to_point2d();
        assert_eq!(projected.x(), 1.0);
        assert_eq!(projected.y(), 2.0);

        // XY平面距離
        let xy_distance = p1.xy_distance_to(&p2);
        assert!((xy_distance - (32.0_f64).sqrt()).abs() < f64::TOLERANCE);
    }

    #[test]
    fn test_point3d_basic_operations_f32() {
        let p1 = Point3D::<f32>::new(1.0, 2.0, 3.0);
        let vec = Vector3D::new(1.0, 1.0, 1.0);

        // 平行移動
        let p2 = p1 + vec;
        assert_eq!(p2.x(), 2.0);
        assert_eq!(p2.y(), 3.0);
        assert_eq!(p2.z(), 4.0);

        // 原点
        let origin = Point3D::<f32>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_point_vector_consistency() {
        // Point + Vector = Point のテスト
        let p = Point2D::<f64>::new(1.0, 2.0);
        let v = Vector2D::new(3.0, 4.0);
        let result = p + v;
        
        // 手動計算と一致するか
        assert_eq!(result.x(), 4.0);
        assert_eq!(result.y(), 6.0);

        // 逆変換も確認
        let back = result - v;
        assert_eq!(back.x(), p.x());
        assert_eq!(back.y(), p.y());
    }
}