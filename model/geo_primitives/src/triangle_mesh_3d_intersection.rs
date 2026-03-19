//! TriangleMesh3D 交差計算実装
//!
//! BasicIntersection トレイトの実装

use crate::{Point3D, TriangleMesh3D};
use geo_contracts::BasicIntersection;
use geo_contracts::Scalar;

// ============================================================================
// BasicIntersection Implementations
// ============================================================================

// TriangleMesh3D vs Point3D
impl<T: Scalar> BasicIntersection<T, Point3D<T>> for TriangleMesh3D<T> {
    type Point = Point3D<T>;

    fn intersection_with(&self, point: &Point3D<T>, tolerance: T) -> Option<Self::Point> {
        // いずれかの三角形上に点がある場合、その点を返す
        for i in 0..self.triangle_count() {
            if let Some(triangle) = self.triangle(i) {
                if triangle.distance_to_point(point) <= tolerance {
                    return Some(*point);
                }
            }
        }

        None
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::BasicIntersection;

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
    fn test_triangle_mesh_intersection_with_point_on_surface() {
        let mesh = create_simple_quad_mesh();

        // メッシュ上の点
        let point_on_mesh = Point3D::new(0.5, 0.5, 0.0);
        let intersection = mesh.intersection_with(&point_on_mesh, 1e-6);
        assert!(intersection.is_some());
        assert_eq!(intersection.unwrap(), point_on_mesh);
    }

    #[test]
    fn test_triangle_mesh_intersection_with_point_outside() {
        let mesh = create_simple_quad_mesh();

        // メッシュから離れた点
        let point_outside = Point3D::new(5.0, 5.0, 5.0);
        let intersection = mesh.intersection_with(&point_outside, 1e-6);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_triangle_mesh_intersection_tolerance() {
        let mesh = create_simple_quad_mesh();

        // メッシュに近い点（許容誤差内）
        let near_point = Point3D::new(0.5, 0.5, 0.05);
        assert!(mesh.intersection_with(&near_point, 0.1).is_some());
        assert!(mesh.intersection_with(&near_point, 0.01).is_none());
    }

    #[test]
    fn test_triangle_mesh_intersection_empty() {
        let empty_mesh = TriangleMesh3D::<f64>::empty();

        let point = Point3D::new(1.0, 1.0, 1.0);
        let intersection = empty_mesh.intersection_with(&point, 1.0);
        assert!(intersection.is_none());
    }

    #[test]
    fn test_triangle_mesh_intersection_at_vertex() {
        let mesh = create_simple_quad_mesh();

        // 頂点位置の点
        let vertex_point = Point3D::new(0.0, 0.0, 0.0);
        let intersection = mesh.intersection_with(&vertex_point, 1e-6);
        assert!(intersection.is_some());
        assert_eq!(intersection.unwrap(), vertex_point);
    }

    #[test]
    fn test_triangle_mesh_intersection_at_edge() {
        let mesh = create_simple_quad_mesh();

        // エッジ上の点
        let edge_point = Point3D::new(0.5, 0.0, 0.0);
        let intersection = mesh.intersection_with(&edge_point, 1e-6);
        assert!(intersection.is_some());
        assert_eq!(intersection.unwrap(), edge_point);
    }
}
