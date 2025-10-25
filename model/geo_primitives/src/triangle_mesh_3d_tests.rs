//! TriangleMesh3D のテスト

use crate::{Point3D, TriangleMesh3D};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_creation() {
        // 三角形1つのメッシュ
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices);
        assert!(mesh.is_ok());

        let mesh = mesh.unwrap();
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
        assert!(mesh.is_valid());
    }

    #[test]
    fn test_invalid_indices() {
        let vertices = vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)];
        // 無効なインデックス（3つの頂点しかないのに3を参照）
        let indices = vec![[0, 1, 3]];

        let mesh = TriangleMesh3D::new(vertices, indices);
        assert!(mesh.is_err());
    }

    #[test]
    fn test_empty_mesh() {
        let mesh = TriangleMesh3D::<f64>::empty();
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
        assert!(mesh.is_valid());
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_mesh_access() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2], [1, 3, 2]];

        let mesh = TriangleMesh3D::new(vertices.clone(), indices.clone()).unwrap();

        // 頂点アクセス
        assert_eq!(mesh.vertex(0), Some(vertices[0]));
        assert_eq!(mesh.vertex(3), Some(vertices[3]));
        assert_eq!(mesh.vertex(4), None);

        // インデックスアクセス
        assert_eq!(mesh.triangle_indices(0), Some([0, 1, 2]));
        assert_eq!(mesh.triangle_indices(1), Some([1, 3, 2]));
        assert_eq!(mesh.triangle_indices(2), None);

        // 配列アクセス
        assert_eq!(mesh.vertices(), &vertices);
        assert_eq!(mesh.indices(), &indices);
    }

    #[test]
    fn test_triangle_extraction() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices.clone(), indices).unwrap();
        let triangle = mesh.triangle(0);

        assert!(triangle.is_some());
        let triangle = triangle.unwrap();
        assert_eq!(triangle.vertex_a(), vertices[0]);
        assert_eq!(triangle.vertex_b(), vertices[1]);
        assert_eq!(triangle.vertex_c(), vertices[2]);
    }

    #[test]
    fn test_degenerate_triangles() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        // 退化した三角形（同じ頂点を2回参照）
        let indices = vec![[0, 1, 2], [0, 0, 1]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        assert_eq!(mesh.degenerate_triangle_count(), 1);
    }

    #[test]
    fn test_bounding_box() {
        let vertices = vec![
            Point3D::new(-1.0, -2.0, -3.0),
            Point3D::new(2.0, 1.0, 0.0),
            Point3D::new(0.0, 3.0, 1.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let bbox = mesh.bounding_box();

        assert!(bbox.is_some());
        let (min_point, max_point) = bbox.unwrap();
        assert_eq!(min_point, Point3D::new(-1.0, -2.0, -3.0));
        assert_eq!(max_point, Point3D::new(2.0, 3.0, 1.0));
    }

    #[test]
    fn test_empty_bounding_box() {
        let mesh = TriangleMesh3D::<f64>::empty();
        assert!(mesh.bounding_box().is_none());
    }

    #[test]
    fn test_quad_mesh() {
        // 四角形を2つの三角形で構成
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0), // 0
            Point3D::new(1.0, 0.0, 0.0), // 1
            Point3D::new(1.0, 1.0, 0.0), // 2
            Point3D::new(0.0, 1.0, 0.0), // 3
        ];
        let indices = vec![
            [0, 1, 2], // 下三角形
            [0, 2, 3], // 上三角形
        ];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
        assert!(mesh.is_valid());
        assert_eq!(mesh.degenerate_triangle_count(), 0);
    }

    #[test]
    fn test_mesh_display() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let display_str = format!("{}", mesh);

        assert!(display_str.contains("TriangleMesh3D"));
        assert!(display_str.contains("3 vertices"));
        assert!(display_str.contains("1 triangles"));
    }
}
