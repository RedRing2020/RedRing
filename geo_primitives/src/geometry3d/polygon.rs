/// 3D多角形プリミティブの定義

use geo_core::{Point3D, Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::geometry_utils::*;

/// 3D多角形プリミティブ（平面多角形）
#[derive(Debug, Clone)]
pub struct Polygon3D {
    vertices: Vec<Point3D>,
}

impl Polygon3D {
    /// 新しい3D多角形を作成
    pub fn new(vertices: Vec<Point3D>) -> Option<Self> {
        if vertices.len() < 3 {
            return None; // 多角形は最低3つの頂点が必要
        }
        Some(Self { vertices })
    }

    /// 頂点を取得
    pub fn vertices(&self) -> &[Point3D] {
        &self.vertices
    }

    /// 頂点の可変参照を取得
    pub fn vertices_mut(&mut self) -> &mut Vec<Point3D> {
        &mut self.vertices
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point3D {
        point3d_centroid(&self.vertices).unwrap()
    }

    /// 法線ベクトルを計算（最初の3点から）
    pub fn normal(&self) -> Option<Vector3D> {
        if self.vertices.len() < 3 {
            return None;
        }

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

        // 長さチェック
        let length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
        if length < 1e-10 {
            return None;
        }

        Some(Vector3D::new(
            Scalar::new(cross_x),
            Scalar::new(cross_y),
            Scalar::new(cross_z),
        ))
    }

    /// 面積を計算（三角形分割を使用）
    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }

        let mut total_area = 0.0;
        let (x0, y0, z0) = point3d_to_f64(&self.vertices[0]);

        for i in 1..n - 1 {
            let (x1, y1, z1) = point3d_to_f64(&self.vertices[i]);
            let (x2, y2, z2) = point3d_to_f64(&self.vertices[i + 1]);

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

            let triangle_area = 0.5 * (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
            total_area += triangle_area;
        }

        total_area
    }
}

impl GeometricPrimitive for Polygon3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Polygon
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
    fn test_polygon3d_creation() {
        let vertices = vec![
            Point3D::from_f64(0.0, 0.0, 0.0),
            Point3D::from_f64(1.0, 0.0, 0.0),
            Point3D::from_f64(1.0, 1.0, 0.0),
            Point3D::from_f64(0.0, 1.0, 0.0),
        ];
        let polygon = Polygon3D::new(vertices).unwrap();
        assert_eq!(polygon.vertex_count(), 4);
    }

    #[test]
    fn test_polygon3d_area() {
        let vertices = vec![
            Point3D::from_f64(0.0, 0.0, 0.0),
            Point3D::from_f64(1.0, 0.0, 0.0),
            Point3D::from_f64(1.0, 1.0, 0.0),
            Point3D::from_f64(0.0, 1.0, 0.0),
        ];
        let polygon = Polygon3D::new(vertices).unwrap();
        assert!((polygon.area() - 1.0).abs() < 1e-10);
    }
}
