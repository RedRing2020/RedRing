/// Phase 1: 最小限のアダプター実装
/// 
/// まずは基本的なLineSegment3Dアダプターのみを実装し、
/// 既存のCurve3D traitとの互換性を確保

use std::any::Any;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use crate::geometry::geometry3d::{point::Point, vector::Vector};
use geo_core::{
    primitives3d::{LineSegment3D as GeoLineSegment3D, ParametricCurve3D, Point3D as GeoPoint3D},
    ToleranceContext,
    Scalar
};

/// model::Point と geo_core::Point3D の変換ユーティリティ
pub struct TypeConverter;

impl TypeConverter {
    /// model::Point から geo_core::Point3D への変換
    pub fn point_to_geo(point: &Point) -> GeoPoint3D {
        GeoPoint3D::from_f64(point.x(), point.y(), point.z())
    }

    /// geo_core::Point3D から model::Point への変換
    pub fn point_from_geo(geo_point: &GeoPoint3D) -> Point {
        Point::new(
            geo_point.x().value(),
            geo_point.y().value(),
            geo_point.z().value()
        )
    }

    /// model::Vector から geo_core::Vector3D への変換
    pub fn vector_to_geo(vector: &Vector) -> geo_core::Vector3D {
        geo_core::Vector3D::new(
            Scalar::new(vector.x()),
            Scalar::new(vector.y()),
            Scalar::new(vector.z())
        )
    }

    /// geo_core::Vector3D から model::Vector への変換
    pub fn vector_from_geo(geo_vector: &geo_core::Vector3D) -> Vector {
        Vector::new(
            geo_vector.x().value(),
            geo_vector.y().value(),
            geo_vector.z().value()
        )
    }
}

/// geo_core LineSegment3D のシンプルなアダプター
#[derive(Debug, Clone)]
pub struct SimpleAdaptedLine {
    inner: GeoLineSegment3D,
    tolerance: ToleranceContext,
}

impl SimpleAdaptedLine {
    pub fn new(start: Point, end: Point) -> Self {
        let geo_start = TypeConverter::point_to_geo(&start);
        let geo_end = TypeConverter::point_to_geo(&end);
        
        Self {
            inner: GeoLineSegment3D::new(geo_start, geo_end),
            tolerance: ToleranceContext::standard(),
        }
    }

    pub fn start(&self) -> Point {
        TypeConverter::point_from_geo(self.inner.start())
    }

    pub fn end(&self) -> Point {
        TypeConverter::point_from_geo(self.inner.end())
    }

    pub fn direction(&self) -> Vector {
        TypeConverter::vector_from_geo(&self.inner.direction())
    }

    /// geo_coreの高精度計算を利用した距離計算
    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let geo_point = TypeConverter::point_to_geo(point);
        self.inner.distance_to_point(&geo_point).value()
    }
}

impl Curve3D for SimpleAdaptedLine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Line
    }

    fn evaluate(&self, t: f64) -> Point {
        let geo_point = self.inner.evaluate(Scalar::new(t));
        TypeConverter::point_from_geo(&geo_point)
    }

    fn derivative(&self, _t: f64) -> Vector {
        let geo_vector = self.inner.derivative(Scalar::new(_t));
        TypeConverter::vector_from_geo(&geo_vector)
    }

    fn length(&self) -> f64 {
        self.inner.length().value()
    }

    fn parameter_hint(&self, pt: &Point) -> f64 {
        // 簡単な最近点パラメータ計算
        let start = self.start();
        let end = self.end();
        let line_vec = Vector::new(end.x() - start.x(), end.y() - start.y(), end.z() - start.z());
        let point_vec = Vector::new(pt.x() - start.x(), pt.y() - start.y(), pt.z() - start.z());
        
        let line_length_sq = line_vec.dot(&line_vec);
        if line_length_sq < 1e-12 {
            return 0.0;
        }
        
        let t = point_vec.dot(&line_vec) / line_length_sq;
        t.clamp(0.0, 1.0)
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// シンプルなファクトリー関数
pub mod simple_factory {
    use super::*;

    pub fn create_line(start: Point, end: Point) -> Box<dyn Curve3D> {
        Box::new(SimpleAdaptedLine::new(start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_adapted_line() {
        let start = Point::new(0.0, 0.0, 0.0);
        let end = Point::new(1.0, 0.0, 0.0);
        let line = SimpleAdaptedLine::new(start, end);

        // Curve3D trait のテスト
        assert_eq!(line.kind(), CurveKind3D::Line);
        
        let mid_point = line.evaluate(0.5);
        assert!((mid_point.x() - 0.5).abs() < 1e-10);
        assert!((mid_point.y() - 0.0).abs() < 1e-10);
        assert!((mid_point.z() - 0.0).abs() < 1e-10);

        let length = line.length();
        assert!((length - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_type_converter() {
        let model_point = Point::new(1.0, 2.0, 3.0);
        let geo_point = TypeConverter::point_to_geo(&model_point);
        let converted_back = TypeConverter::point_from_geo(&geo_point);

        assert!((converted_back.x() - model_point.x()).abs() < 1e-10);
        assert!((converted_back.y() - model_point.y()).abs() < 1e-10);
        assert!((converted_back.z() - model_point.z()).abs() < 1e-10);
    }

    #[test]
    fn test_curve3d_downcasting() {
        let line: Box<dyn Curve3D> = simple_factory::create_line(
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0)
        );

        // Any downcasting テスト
        let adapted_line = line.as_any().downcast_ref::<SimpleAdaptedLine>();
        assert!(adapted_line.is_some());
    }
}