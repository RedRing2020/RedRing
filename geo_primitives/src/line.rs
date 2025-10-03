/// 線分プリミティブの定義
/// 
/// 2D/3D空間における線分要素

use geo_core::{Vector2D, Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};
use crate::point::{Point2D, Point3D};

/// 2D線分プリミティブ
#[derive(Debug, Clone)]
pub struct LineSegment2D {
    start: Point2D,
    end: Point2D,
}

impl LineSegment2D {
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }
    
    pub fn start(&self) -> &Point2D {
        &self.start
    }
    
    pub fn end(&self) -> &Point2D {
        &self.end
    }
    
    pub fn direction(&self) -> Vector2D {
        Vector2D::new(
            Scalar::new(self.end.x() - self.start.x()),
            Scalar::new(self.end.y() - self.start.y()),
        )
    }
    
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }
    
    pub fn midpoint(&self) -> Point2D {
        Point2D::new(
            (self.start.x() + self.end.x()) / 2.0,
            (self.start.y() + self.end.y()) / 2.0,
        )
    }
    
    /// パラメータt [0,1] で線分上の点を取得
    pub fn evaluate(&self, t: f64) -> Point2D {
        let t = t.clamp(0.0, 1.0);
        Point2D::new(
            self.start.x() + t * (self.end.x() - self.start.x()),
            self.start.y() + t * (self.end.y() - self.start.y()),
        )
    }
}

impl GeometricPrimitive for LineSegment2D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
    
    fn bounding_box(&self) -> BoundingBox {
        let min_x = self.start.x().min(self.end.x());
        let min_y = self.start.y().min(self.end.y());
        let max_x = self.start.x().max(self.end.x());
        let max_y = self.start.y().max(self.end.y());
        
        BoundingBox::from_2d(
            geo_core::Point2D::from_f64(min_x, min_y),
            geo_core::Point2D::from_f64(max_x, max_y),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.length())
    }
}

/// 3D線分プリミティブ
#[derive(Debug, Clone)]
pub struct LineSegment3D {
    start: Point3D,
    end: Point3D,
}

impl LineSegment3D {
    pub fn new(start: Point3D, end: Point3D) -> Self {
        Self { start, end }
    }
    
    pub fn start(&self) -> &Point3D {
        &self.start
    }
    
    pub fn end(&self) -> &Point3D {
        &self.end
    }
    
    pub fn direction(&self) -> Vector3D {
        Vector3D::new(
            Scalar::new(self.end.x() - self.start.x()),
            Scalar::new(self.end.y() - self.start.y()),
            Scalar::new(self.end.z() - self.start.z()),
        )
    }
    
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }
    
    pub fn midpoint(&self) -> Point3D {
        Point3D::new(
            (self.start.x() + self.end.x()) / 2.0,
            (self.start.y() + self.end.y()) / 2.0,
            (self.start.z() + self.end.z()) / 2.0,
        )
    }
    
    /// パラメータt [0,1] で線分上の点を取得
    pub fn evaluate(&self, t: f64) -> Point3D {
        let t = t.clamp(0.0, 1.0);
        Point3D::new(
            self.start.x() + t * (self.end.x() - self.start.x()),
            self.start.y() + t * (self.end.y() - self.start.y()),
            self.start.z() + t * (self.end.z() - self.start.z()),
        )
    }
}

impl GeometricPrimitive for LineSegment3D {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::LineSegment
    }
    
    fn bounding_box(&self) -> BoundingBox {
        let min_x = self.start.x().min(self.end.x());
        let min_y = self.start.y().min(self.end.y());
        let min_z = self.start.z().min(self.end.z());
        let max_x = self.start.x().max(self.end.x());
        let max_y = self.start.y().max(self.end.y());
        let max_z = self.start.z().max(self.end.z());
        
        BoundingBox::new(
            geo_core::Point3D::from_f64(min_x, min_y, min_z),
            geo_core::Point3D::from_f64(max_x, max_y, max_z),
        )
    }
    
    fn measure(&self) -> Option<f64> {
        Some(self.length())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_segment_2d_creation() {
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(3.0, 4.0);
        let line = LineSegment2D::new(start, end);
        
        assert_eq!(line.length(), 5.0);
        assert_eq!(line.primitive_kind(), PrimitiveKind::LineSegment);
    }

    #[test]
    fn test_line_segment_3d_evaluation() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(2.0, 2.0, 2.0);
        let line = LineSegment3D::new(start, end);
        
        let midpoint = line.evaluate(0.5);
        assert_eq!(midpoint.x(), 1.0);
        assert_eq!(midpoint.y(), 1.0);
        assert_eq!(midpoint.z(), 1.0);
    }

    #[test]
    fn test_line_segment_bounding_box() {
        let start = Point3D::new(1.0, 2.0, 3.0);
        let end = Point3D::new(4.0, 1.0, 5.0);
        let line = LineSegment3D::new(start, end);
        
        let bbox = line.bounding_box();
        assert_eq!(bbox.min.x().value(), 1.0);
        assert_eq!(bbox.min.y().value(), 1.0);
        assert_eq!(bbox.max.z().value(), 5.0);
    }
}