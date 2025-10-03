/// 多角形プリミティブの定義
///
/// 2D/3D空間における多角形要素

use geo_core::{Vector3D, Scalar, Vector};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::{Point2D, Point3D};

/// 2D多角形プリミティブ
#[derive(Debug, Clone)]
pub struct Polygon2D {
    vertices: Vec<Point2D>,
    is_closed: bool,
}

impl Polygon2D {
    /// 頂点列から多角形を作成
    pub fn new(vertices: Vec<Point2D>) -> Option<Self> {
        if vertices.len() < 3 {
            None
        } else {
            Some(Self {
                vertices,
                is_closed: true
            })
        }
    }

    /// 開いた多角形（ポリライン）を作成
    pub fn new_open(vertices: Vec<Point2D>) -> Option<Self> {
        if vertices.len() < 2 {
            None
        } else {
            Some(Self {
                vertices,
                is_closed: false
            })
        }
    }

    pub fn vertices(&self) -> &Vec<Point2D> {
        &self.vertices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point2D {
        let sum_x: f64 = self.vertices.iter().map(|v| v.x()).sum();
        let sum_y: f64 = self.vertices.iter().map(|v| v.y()).sum();
        let count = self.vertices.len() as f64;

        Point2D::new(sum_x / count, sum_y / count)
    }

    /// 面積を計算（閉じた多角形のみ）
    pub fn area(&self) -> f64 {
        if !self.is_closed || self.vertices.len() < 3 {
            return 0.0;
        }

        let mut area = 0.0;
        let n = self.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area += self.vertices[i].x() * self.vertices[j].y();
            area -= self.vertices[j].x() * self.vertices[i].y();
        }

        0.5 * area.abs()
    }

    /// 周囲長を計算
    pub fn perimeter(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }

        let mut perimeter = 0.0;
        let n = self.vertices.len();

        for i in 0..(n - 1) {
            perimeter += self.vertices[i].distance_to(&self.vertices[i + 1]);
        }

        // 閉じた多角形の場合は最後の辺を追加
        if self.is_closed && n >= 3 {
            perimeter += self.vertices[n - 1].distance_to(&self.vertices[0]);
        }

        perimeter
    }
}

impl GeometricPrimitive for Polygon2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        if self.is_closed {
            PrimitiveKind::Polygon
        } else {
            PrimitiveKind::PolyLine
        }
    }

    fn bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            return BoundingBox::from_2d(
                geo_core::Point2D::from_f64(0.0, 0.0),
                geo_core::Point2D::from_f64(0.0, 0.0),
            );
        }

        let min_x = self.vertices.iter().map(|v| v.x()).fold(f64::INFINITY, f64::min);
        let min_y = self.vertices.iter().map(|v| v.y()).fold(f64::INFINITY, f64::min);
        let max_x = self.vertices.iter().map(|v| v.x()).fold(f64::NEG_INFINITY, f64::max);
        let max_y = self.vertices.iter().map(|v| v.y()).fold(f64::NEG_INFINITY, f64::max);

        BoundingBox::from_2d(
            geo_core::Point2D::from_f64(min_x, min_y),
            geo_core::Point2D::from_f64(max_x, max_y),
        )
    }

    fn measure(&self) -> Option<f64> {
        if self.is_closed {
            Some(self.area())
        } else {
            Some(self.perimeter()) // ポリラインの場合は長さ
        }
    }
}

/// 3D多角形プリミティブ
#[derive(Debug, Clone)]
pub struct Polygon3D {
    vertices: Vec<Point3D>,
    is_closed: bool,
}

impl Polygon3D {
    /// 頂点列から多角形を作成
    pub fn new(vertices: Vec<Point3D>) -> Option<Self> {
        if vertices.len() < 3 {
            None
        } else {
            Some(Self {
                vertices,
                is_closed: true
            })
        }
    }

    pub fn vertices(&self) -> &Vec<Point3D> {
        &self.vertices
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point3D {
        let sum_x: f64 = self.vertices.iter().map(|v| v.x()).sum();
        let sum_y: f64 = self.vertices.iter().map(|v| v.y()).sum();
        let sum_z: f64 = self.vertices.iter().map(|v| v.z()).sum();
        let count = self.vertices.len() as f64;

        Point3D::new(sum_x / count, sum_y / count, sum_z / count)
    }

    /// 法線ベクトルを計算（最初の3頂点から）
    pub fn normal(&self) -> Option<Vector3D> {
        if self.vertices.len() < 3 {
            return None;
        }

        let v1 = Vector3D::new(
            Scalar::new(self.vertices[1].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[1].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[1].z() - self.vertices[0].z()),
        );
        let v2 = Vector3D::new(
            Scalar::new(self.vertices[2].x() - self.vertices[0].x()),
            Scalar::new(self.vertices[2].y() - self.vertices[0].y()),
            Scalar::new(self.vertices[2].z() - self.vertices[0].z()),
        );

        let cross = v1.cross(&v2);
        let mag_squared = cross.x().value() * cross.x().value()
            + cross.y().value() * cross.y().value()
            + cross.z().value() * cross.z().value();

        if mag_squared > 1e-20 {
            let magnitude = Scalar::new(mag_squared.sqrt());
            Some(Vector3D::new(
                cross.x() / magnitude,
                cross.y() / magnitude,
                cross.z() / magnitude,
            ))
        } else {
            None
        }
    }

    /// 面積を計算（平面多角形の場合）
    pub fn area(&self) -> f64 {
        if !self.is_closed || self.vertices.len() < 3 {
            return 0.0;
        }

        let normal = match self.normal() {
            Some(n) => n,
            None => return 0.0, // 縮退多角形
        };

        let mut area = 0.0;
        let n = self.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let v1 = Vector3D::new(
                Scalar::new(self.vertices[i].x()),
                Scalar::new(self.vertices[i].y()),
                Scalar::new(self.vertices[i].z()),
            );
            let v2 = Vector3D::new(
                Scalar::new(self.vertices[j].x()),
                Scalar::new(self.vertices[j].y()),
                Scalar::new(self.vertices[j].z()),
            );

            let cross = v1.cross(&v2);
            // dot積を手動で計算
            let dot_value = cross.x().value() * normal.x().value()
                + cross.y().value() * normal.y().value()
                + cross.z().value() * normal.z().value();
            area += dot_value;
        }

        0.5 * area.abs()
    }
}

impl GeometricPrimitive for Polygon3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Polygon
    }

    fn bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            return BoundingBox::new(
                geo_core::Point3D::from_f64(0.0, 0.0, 0.0),
                geo_core::Point3D::from_f64(0.0, 0.0, 0.0),
            );
        }

        let min_x = self.vertices.iter().map(|v| v.x()).fold(f64::INFINITY, f64::min);
        let min_y = self.vertices.iter().map(|v| v.y()).fold(f64::INFINITY, f64::min);
        let min_z = self.vertices.iter().map(|v| v.z()).fold(f64::INFINITY, f64::min);
        let max_x = self.vertices.iter().map(|v| v.x()).fold(f64::NEG_INFINITY, f64::max);
        let max_y = self.vertices.iter().map(|v| v.y()).fold(f64::NEG_INFINITY, f64::max);
        let max_z = self.vertices.iter().map(|v| v.z()).fold(f64::NEG_INFINITY, f64::max);

        BoundingBox::new(
            geo_core::Point3D::from_f64(min_x, min_y, min_z),
            geo_core::Point3D::from_f64(max_x, max_y, max_z),
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
    fn test_polygon_2d_area() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();

        assert!((polygon.area() - 1.0).abs() < 1e-10);
        assert_eq!(polygon.primitive_kind(), PrimitiveKind::Polygon);
    }

    #[test]
    fn test_polygon_3d_creation() {
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.5, 1.0, 0.0),
        ];
        let polygon = Polygon3D::new(vertices).unwrap();

        assert_eq!(polygon.vertex_count(), 3);
        assert!(polygon.is_closed());

        let normal = polygon.normal().unwrap();
        // Z軸方向の法線になるはず
        assert!((normal.z().value() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_polygon_2d_perimeter() {
        let vertices = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(3.0, 0.0),
            Point2D::new(3.0, 4.0),
            Point2D::new(0.0, 4.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();

        assert!((polygon.perimeter() - 14.0).abs() < 1e-10); // 3+4+3+4 = 14
    }
}
