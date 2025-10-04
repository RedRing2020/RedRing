/// 三角形プリミティブの定義
///
/// 2D/3D空間における三角形要素

use geo_core::Vector3D;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::geometry_utils::*;
use geo_core::{Point2D, Point3D};

/// 2D三角形プリミティブ
#[derive(Debug, Clone)]
pub struct Triangle2D {
    vertices: [Point2D; 3],
}

impl Triangle2D {
    /// 新しい2D三角形を作成
    pub fn new(v0: Point2D, v1: Point2D, v2: Point2D) -> Self {
        Self {
            vertices: [v0, v1, v2],
        }
    }

    /// 頂点を取得
    pub fn vertices(&self) -> &[Point2D; 3] {
        &self.vertices
    }

    /// 頂点の可変参照を取得
    pub fn vertices_mut(&mut self) -> &mut [Point2D; 3] {
        &mut self.vertices
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point2D {
        let (x0, y0) = point2d_to_f64(&self.vertices[0]);
        let (x1, y1) = point2d_to_f64(&self.vertices[1]);
        let (x2, y2) = point2d_to_f64(&self.vertices[2]);
        point2d_from_f64(
            (x0 + x1 + x2) / 3.0,
            (y0 + y1 + y2) / 3.0,
        )
    }

    /// 面積を計算
    pub fn area(&self) -> f64 {
        let (x0, y0) = point2d_to_f64(&self.vertices[0]);
        let (x1, y1) = point2d_to_f64(&self.vertices[1]);
        let (x2, y2) = point2d_to_f64(&self.vertices[2]);
        let v1_x = x1 - x0;
        let v1_y = y1 - y0;
        let v2_x = x2 - x0;
        let v2_y = y2 - y0;
        0.5 * (v1_x * v2_y - v1_y * v2_x).abs()
    }

    /// 点が三角形内部にあるかを判定（重心座標系を使用）
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let (x0, y0) = point2d_to_f64(&self.vertices[0]);
        let (x1, y1) = point2d_to_f64(&self.vertices[1]);
        let (x2, y2) = point2d_to_f64(&self.vertices[2]);
        let (px, py) = point2d_to_f64(point);

        let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);
        
        if denom.abs() < 1e-10 {
            return false; // 退化した三角形
        }

        let alpha = ((y1 - y2) * (px - x2) + (x2 - x1) * (py - y2)) / denom;
        let beta = ((y2 - y0) * (px - x2) + (x0 - x2) * (py - y2)) / denom;
        let gamma = 1.0 - alpha - beta;

        alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0
    }
}

impl GeometricPrimitive for Triangle2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }

    fn bounding_box(&self) -> BoundingBox {
        let bbox = point2d_bounding_box(&self.vertices).unwrap();
        BoundingBox::from_2d(
            point2d_from_f64(bbox.0, bbox.1),
            point2d_from_f64(bbox.2, bbox.3),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.area())
    }
}

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
        
        Vector3D::from_f64(cross_x, cross_y, cross_z)
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
    fn test_triangle2d_creation() {
        let p1 = Point2D::from_f64(0.0, 0.0);
        let p2 = Point2D::from_f64(1.0, 0.0);
        let p3 = Point2D::from_f64(0.0, 1.0);
        let triangle = Triangle2D::new(p1, p2, p3);
        assert_eq!(triangle.vertices().len(), 3);
    }

    #[test]
    fn test_triangle2d_area() {
        let p1 = Point2D::from_f64(0.0, 0.0);
        let p2 = Point2D::from_f64(1.0, 0.0);
        let p3 = Point2D::from_f64(0.0, 1.0);
        let triangle = Triangle2D::new(p1, p2, p3);
        assert!((triangle.area() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_triangle2d_centroid() {
        let p1 = Point2D::from_f64(0.0, 0.0);
        let p2 = Point2D::from_f64(3.0, 0.0);
        let p3 = Point2D::from_f64(0.0, 3.0);
        let triangle = Triangle2D::new(p1, p2, p3);
        let centroid = triangle.centroid();
        let (cx, cy) = point2d_to_f64(&centroid);
        assert!((cx - 1.0).abs() < 1e-10);
        assert!((cy - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_triangle2d_contains_point() {
        let p1 = Point2D::from_f64(0.0, 0.0);
        let p2 = Point2D::from_f64(2.0, 0.0);
        let p3 = Point2D::from_f64(1.0, 2.0);
        let triangle = Triangle2D::new(p1, p2, p3);
        
        let inside_point = Point2D::from_f64(1.0, 0.5);
        let outside_point = Point2D::from_f64(2.0, 2.0);
        
        assert!(triangle.contains_point(&inside_point));
        assert!(!triangle.contains_point(&outside_point));
    }

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