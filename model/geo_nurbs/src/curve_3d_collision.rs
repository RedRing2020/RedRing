//! NurbsCurve3D - Collision Detection Implementation
//!
//! NURBS 3次元曲線の衝突判定実装
//!
//! 本実装は段階的に精度を向上させるアプローチを採用：
//! 1. BBox による事前スクリーニング
//! 2. 離散化による近似計算
//! 3. Newton-Raphson法による精密計算（将来実装）

use crate::{NurbsCurve3D, Scalar};
use geo_core::Point3D;
use geo_foundation::extensions::BasicCollision;

// ============================================================================
// NurbsCurve3D vs Point3D
// ============================================================================

impl<T: Scalar> BasicCollision<T, Point3D<T>> for NurbsCurve3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 簡易実装: 曲線を離散化して最小距離を計算
        // TODO: Newton-Raphson法による精密計算
        
        let num_samples = 100; // サンプリング数
        let mut min_distance = T::INFINITY;

        // パラメータ範囲を取得
        let (u_min, u_max) = self.parameter_domain();
        let delta_u = (u_max - u_min) / T::from_usize(num_samples);

        // 曲線上の点をサンプリングして最小距離を計算
        for i in 0..=num_samples {
            let u = u_min + delta_u * T::from_usize(i);
            
            let curve_point = self.evaluate_at(u);
            
            // 曲線上の点とターゲット点の距離
            let dx = curve_point.x() - point.x();
            let dy = curve_point.y() - point.y();
            let dz = curve_point.z() - point.z();
            let distance = (dx * dx + dy * dy + dz * dz).sqrt();
            
            if distance < min_distance {
                min_distance = distance;
            }
        }

        min_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NurbsCurve3D;
    use analysis::linalg::vector::Vector3;

    const TOLERANCE: f64 = 1e-6;

    /// テスト用のシンプルなNURBS曲線を作成（線分）
    fn create_line_curve() -> NurbsCurve3D<f64> {
        let control_points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        ];
        let knot_vector = vec![0.0, 0.0, 1.0, 1.0];
        
        NurbsCurve3D::new(control_points, None, knot_vector, 1).unwrap()
    }

    #[test]
    fn test_point_on_curve() {
        let curve = create_line_curve();
        let point = Point3D::new(0.5, 0.0, 0.0);

        assert!(curve.intersects(&point, TOLERANCE));
        let distance = curve.distance_to(&point);
        assert!(distance < TOLERANCE, "Distance: {}", distance);
    }

    #[test]
    fn test_point_near_curve() {
        let curve = create_line_curve();
        let point = Point3D::new(0.5, 0.1, 0.0);

        let distance = curve.distance_to(&point);
        assert!((distance - 0.1).abs() < TOLERANCE * 10.0, "Distance: {}", distance);
    }

    #[test]
    fn test_point_far_from_curve() {
        let curve = create_line_curve();
        let point = Point3D::new(5.0, 5.0, 5.0);

        assert!(!curve.intersects(&point, TOLERANCE));
        let distance = curve.distance_to(&point);
        assert!(distance > 1.0);
    }
}
