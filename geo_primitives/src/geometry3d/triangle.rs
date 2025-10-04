/// 3D三角形プリミティブの定義

use geo_core::{Point3D, Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::geometry_utils::*;

/// 3D三角形プリミティブ
#[derive(Debug, Clone)]
pub struct Triangle3D {
    vertices: [Point3D; 3],
}

impl Triangle3D {
    /// 新しい3D三角形を作成
    pub fn new(v0: Point3D, v1: Point3D, v2: Point3D) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }

    /// 頂点を取得
    pub fn vertices(&self) -> &[Point3D; 3] {
        &self.vertices
    }

    /// 頂点の可変参照を取得
    pub fn vertices_mut(&mut self) -> &mut [Point3D; 3] {
        &mut self.vertices
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point3D {
        point3d_centroid(&self.vertices).unwrap()
    }

    /// 面積を計算（外積を使用）
    pub fn area(&self) -> f64 {
        let (x0, y0, z0) = point3d_to_f64(&self.vertices[0]);
        let (x1, y1, z1) = point3d_to_f64(&self.vertices[1]);
        let (x2, y2, z2) = point3d_to_f64(&self.vertices[2]);

        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v1_z = z1 - z0;

        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        let v2_z = z2 - z0;

        // 外積
        let cross_x = v1_y * v2_z - v1_z * v2_y;
        let cross_y = v1_z * v2_x - v1_x * v2_z;
        let cross_z = v1_x * v2_y - v1_y * v2_x;

        0.5 * (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt()
    }

    /// 法線ベクトルを計算
    pub fn normal(&self) -> Vector3D {
        let (x0, y0, z0) = point3d_to_f64(&self.vertices[0]);
        let (x1, y1, z1) = point3d_to_f64(&self.vertices[1]);
        let (x2, y2, z2) = point3d_to_f64(&self.vertices[2]);

        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v1_z = z1 - z0;

        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        let v2_z = z2 - z0;

        // 外積
        let cross_x = v1_y * v2_z - v1_z * v2_y;
        let cross_y = v1_z * v2_x - v1_x * v2_z;
        let cross_z = v1_x * v2_y - v1_y * v2_x;

        Vector3D::new(
            Scalar::new(cross_x),
            Scalar::new(cross_y),
            Scalar::new(cross_z),
        )
    }
}

impl GeometricPrimitive for Triangle3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }

    fn bounding_box(&self) -> BoundingBox {
        let bbox = point3d_bounding_box(&self.vertices).unwrap();
        BoundingBox::new(
            point3d_from_f64(bbox.0, bbox.1, bbox.2),
            point3d_from_f64(bbox.3, bbox.4, bbox.5),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle3d_creation() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);
        let triangle = Triangle3D::new(p1, p2, p3);
        assert_eq!(triangle.vertices().len(), 3);
    }

    #[test]
    fn test_triangle3d_area() {
        let p1 = Point3D::from_f64(0.0, 0.0, 0.0);
        let p2 = Point3D::from_f64(1.0, 0.0, 0.0);
        let p3 = Point3D::from_f64(0.0, 1.0, 0.0);
        let triangle = Triangle3D::new(p1, p2, p3);
        assert!((triangle.area() - 0.5).abs() < 1e-10);
    }
}
