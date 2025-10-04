/// 2D多角形プリミティブの定義

use geo_core::Point2D;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::geometry_utils::*;

/// 2D多角形プリミティブ
#[derive(Debug, Clone)]
pub struct Polygon2D {
    vertices: Vec<Point2D>,
}

impl Polygon2D {
    /// 新しい2D多角形を作成
    pub fn new(vertices: Vec<Point2D>) -> Option<Self> {
        if vertices.len() < 3 {
            return None; // 多角形は最低3つの頂点が必要
        }
        Some(Self { vertices })
    }

    /// 頂点を取得
    pub fn vertices(&self) -> &[Point2D] {
        &self.vertices
    }

    /// 頂点の可変参照を取得
    pub fn vertices_mut(&mut self) -> &mut Vec<Point2D> {
        &mut self.vertices
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 重心を計算
    pub fn centroid(&self) -> Point2D {
        point2d_centroid(&self.vertices).unwrap()
    }

    /// 面積を計算（Shoelace公式を使用）
    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        if n < 3 {
            return 0.0;
        }

        let mut area: f64 = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);
            area += xi * yj;
            area -= xj * yi;
        }

        (area / 2.0).abs()
    }

    /// 周囲長を計算
    pub fn perimeter(&self) -> f64 {
        let n = self.vertices.len();
        if n < 2 {
            return 0.0;
        }

        let mut perimeter = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);
            let dx = xj - xi;
            let dy = yj - yi;
            perimeter += (dx * dx + dy * dy).sqrt();
        }

        perimeter
    }

    /// 点が多角形内部にあるかを判定（Ray casting algorithm）
    pub fn contains_point(&self, point: &Point2D) -> bool {
        let n = self.vertices.len();
        if n < 3 {
            return false;
        }

        let (px, py) = point2d_to_f64(point);
        let mut inside = false;

        for i in 0..n {
            let j = (i + 1) % n;
            let (xi, yi) = point2d_to_f64(&self.vertices[i]);
            let (xj, yj) = point2d_to_f64(&self.vertices[j]);

            if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
                inside = !inside;
            }
        }

        inside
    }
}

impl GeometricPrimitive for Polygon2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Polygon
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polygon2d_creation() {
        let vertices = vec![
            Point2D::from_f64(0.0, 0.0),
            Point2D::from_f64(1.0, 0.0),
            Point2D::from_f64(1.0, 1.0),
            Point2D::from_f64(0.0, 1.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();
        assert_eq!(polygon.vertex_count(), 4);
    }

    #[test]
    fn test_polygon2d_area() {
        let vertices = vec![
            Point2D::from_f64(0.0, 0.0),
            Point2D::from_f64(1.0, 0.0),
            Point2D::from_f64(1.0, 1.0),
            Point2D::from_f64(0.0, 1.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();
        assert!((polygon.area() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_polygon2d_perimeter() {
        let vertices = vec![
            Point2D::from_f64(0.0, 0.0),
            Point2D::from_f64(1.0, 0.0),
            Point2D::from_f64(1.0, 1.0),
            Point2D::from_f64(0.0, 1.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();
        assert!((polygon.perimeter() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_polygon2d_contains_point() {
        let vertices = vec![
            Point2D::from_f64(0.0, 0.0),
            Point2D::from_f64(2.0, 0.0),
            Point2D::from_f64(2.0, 2.0),
            Point2D::from_f64(0.0, 2.0),
        ];
        let polygon = Polygon2D::new(vertices).unwrap();

        let inside_point = Point2D::from_f64(1.0, 1.0);
        let outside_point = Point2D::from_f64(3.0, 3.0);

        assert!(polygon.contains_point(&inside_point));
        assert!(!polygon.contains_point(&outside_point));
    }
}
