//! TriangleMesh3D 衝突検出・距離計算実装
//!
//! BasicCollision トレイトの実装

use crate::{Point3D, TriangleMesh3D};
use geo_foundation::{extensions::BasicCollision, Scalar};

// ============================================================================
// BasicCollision Implementations
// ============================================================================

// TriangleMesh3D vs Point3D
impl<T: Scalar> BasicCollision<T, Point3D<T>> for TriangleMesh3D<T> {
    type Point2D = Point3D<T>;

    fn intersects(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    fn overlaps(&self, point: &Point3D<T>, tolerance: T) -> bool {
        // メッシュは面の集合なので、点との overlap は intersects と同じ
        self.intersects(point, tolerance)
    }

    fn distance_to(&self, point: &Point3D<T>) -> T {
        // 全ての三角形からの最小距離を計算
        if self.is_empty() {
            return T::INFINITY;
        }

        let mut min_distance = T::INFINITY;

        for i in 0..self.triangle_count() {
            if let Some(triangle) = self.triangle(i) {
                let distance = triangle.distance_to_point(point);
                if distance < min_distance {
                    min_distance = distance;
                }
            }
        }

        min_distance
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Triangle3D;
    use geo_foundation::extensions::BasicCollision;

    fn create_simple_quad_mesh() -> TriangleMesh3D<f64> {
        // XY平面上の四角形メッシュ（2つの三角形）
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];

        let indices = vec![[0, 1, 2], [0, 2, 3]];

        TriangleMesh3D::new(vertices, indices).unwrap()
    }

    #[test]
    fn test_triangle_mesh_collision_with_point_on_surface() {
        let mesh = create_simple_quad_mesh();

        // メッシュ上の点
        let point_on_mesh = Point3D::new(0.5, 0.5, 0.0);
        assert!(mesh.intersects(&point_on_mesh, 1e-10));
    }

    #[test]
    fn test_triangle_mesh_collision_with_point_outside() {
        let mesh = create_simple_quad_mesh();

        // メッシュから離れた点
        let point_outside = Point3D::new(5.0, 5.0, 5.0);
        assert!(!mesh.intersects(&point_outside, 1e-10));
    }

    #[test]
    fn test_triangle_mesh_distance() {
        let mesh = create_simple_quad_mesh();

        // メッシュ上の点との距離は0
        let point_on_mesh = Point3D::new(0.5, 0.5, 0.0);
        let distance = mesh.distance_to(&point_on_mesh);
        assert!(distance < 1e-10);

        // Z方向に離れた点
        let point_above = Point3D::new(0.5, 0.5, 1.0);
        let distance_above = mesh.distance_to(&point_above);
        assert!((distance_above - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_triangle_mesh_empty() {
        let empty_mesh = TriangleMesh3D::<f64>::empty();

        let point = Point3D::new(1.0, 1.0, 1.0);
        let distance = empty_mesh.distance_to(&point);
        assert!(distance.is_infinite());
        assert!(!empty_mesh.intersects(&point, 1.0));
    }

    #[test]
    fn test_triangle_mesh_tolerance() {
        let mesh = create_simple_quad_mesh();

        // メッシュに近い点
        let near_point = Point3D::new(0.5, 0.5, 0.1);
        assert!(mesh.intersects(&near_point, 0.2));
        assert!(!mesh.intersects(&near_point, 0.05));
    }

    #[test]
    fn test_triangle_mesh_distance_to_corner() {
        let mesh = create_simple_quad_mesh();

        // コーナー頂点に近い点
        let near_corner = Point3D::new(0.0, 0.0, 1.0);
        let distance = mesh.distance_to(&near_corner);
        assert!((distance - 1.0).abs() < 1e-10);
    }
}
