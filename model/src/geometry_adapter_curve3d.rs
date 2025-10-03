/// geometry_adapter.rs の Curve3D trait 統合拡張
///
/// 既存のCurve3D trait設計を保持しつつ、geo_coreの数値基盤を活用

use std::any::Any;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use super::{Vector3D, Point3D};
use geo_core::{
    primitives3d::{LineSegment3D as GeoLineSegment3D, ParametricCurve3D},
    ToleranceContext,
    Scalar
};

/// geo_core LineSegment3D のアダプター実装
pub struct AdaptedLine {
    inner: GeoLineSegment3D,
    tolerance: ToleranceContext,
}

impl AdaptedLine {
    pub fn new(start: Point3D, end: Point3D) -> Self {
        let geo_start = start.as_geo_core().clone();
        let geo_end = end.as_geo_core().clone();
        Self {
            inner: GeoLineSegment3D::new(geo_start, geo_end),
            tolerance: ToleranceContext::standard(),
        }
    }

    pub fn from_geo_core(inner: GeoLineSegment3D, tolerance: ToleranceContext) -> Self {
        Self { inner, tolerance }
    }
}

impl Curve3D for AdaptedLine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Line
    }

    fn evaluate(&self, t: f64) -> Point3D {
        let geo_point = self.inner.evaluate(Scalar::new(t));
        Point3D::from_geo_core(geo_point)
    }

    fn derivative(&self, _t: f64) -> Vector3D {
        // 直線の微分は定ベクトル
        let direction = self.inner.direction();
        Vector3D::from_geo_core(direction.to_vector())
    }

    fn length(&self) -> f64 {
        self.inner.length().value()
    }

    fn parameter_hint(&self, pt: &Point3D) -> f64 {
        // 最近点のパラメータを計算
        let geo_pt = pt.as_geo_core();
        self.inner.closest_parameter(geo_pt, &self.tolerance).value()
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// geo_core Arc3D のアダプター実装
pub struct AdaptedArc {
    inner: GeoArc3D,
    tolerance: ToleranceContext,
}

impl AdaptedArc {
    pub fn new(center: Point3D, radius: f64, start_angle: f64, end_angle: f64, normal: Vector3D) -> Self {
        let geo_center = center.as_geo_core().clone();
        let geo_normal = normal.as_geo_core().clone();

        Self {
            inner: GeoArc3D::new(
                geo_center,
                Scalar::new(radius),
                Scalar::new(start_angle),
                Scalar::new(end_angle),
                geo_normal
            ),
            tolerance: ToleranceContext::standard(),
        }
    }
}

impl Curve3D for AdaptedArc {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Arc
    }

    fn evaluate(&self, t: f64) -> Point3D {
        let geo_point = self.inner.evaluate(Scalar::new(t));
        Point3D::from_geo_core(geo_point)
    }

    fn derivative(&self, t: f64) -> Vector3D {
        let geo_tangent = self.inner.derivative(Scalar::new(t));
        Vector3D::from_geo_core(geo_tangent)
    }

    fn length(&self) -> f64 {
        self.inner.length().value()
    }

    fn parameter_hint(&self, pt: &Point3D) -> f64 {
        let geo_pt = pt.as_geo_core();
        self.inner.closest_parameter(geo_pt, &self.tolerance).value()
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// ジェネリックパラメトリック曲線アダプター
pub struct AdaptedParametricCurve3D {
    inner: GeoParametricCurve3D,
    curve_kind: CurveKind3D,
    tolerance: ToleranceContext,
}

impl AdaptedParametricCurve3D {
    pub fn new(inner: GeoParametricCurve3D, curve_kind: CurveKind3D) -> Self {
        Self {
            inner,
            curve_kind,
            tolerance: ToleranceContext::standard(),
        }
    }
}

impl Curve3D for AdaptedParametricCurve3D {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        self.curve_kind
    }

    fn evaluate(&self, t: f64) -> Point3D {
        let geo_point = self.inner.evaluate(Scalar::new(t));
        Point3D::from_geo_core(geo_point)
    }

    fn derivative(&self, t: f64) -> Vector3D {
        let geo_tangent = self.inner.derivative(Scalar::new(t));
        Vector3D::from_geo_core(geo_tangent)
    }

    fn length(&self) -> f64 {
        self.inner.length().value()
    }

    fn parameter_hint(&self, pt: &Point3D) -> f64 {
        let geo_pt = pt.as_geo_core();
        self.inner.closest_parameter(geo_pt, &self.tolerance).value()
    }

    fn domain(&self) -> (f64, f64) {
        let (start, end) = self.inner.domain();
        (start.value(), end.value())
    }
}

/// ファクトリー関数群
pub mod curve_factory {
    use super::*;

    pub fn create_line(start: Point3D, end: Point3D) -> Box<dyn Curve3D> {
        Box::new(AdaptedLine::new(start, end))
    }

    pub fn create_arc(center: Point3D, radius: f64, start_angle: f64, end_angle: f64, normal: Vector3D) -> Box<dyn Curve3D> {
        Box::new(AdaptedArc::new(center, radius, start_angle, end_angle, normal))
    }

    pub fn create_parametric_curve(inner: GeoParametricCurve3D, kind: CurveKind3D) -> Box<dyn Curve3D> {
        Box::new(AdaptedParametricCurve3D::new(inner, kind))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapted_line_curve3d() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(1.0, 1.0, 1.0);
        let line = AdaptedLine::new(start, end);

        // Curve3D trait のテスト
        assert_eq!(line.kind(), CurveKind3D::Line);

        let mid_point = line.evaluate(0.5);
        assert!((mid_point.x() - 0.5).abs() < 1e-10);
        assert!((mid_point.y() - 0.5).abs() < 1e-10);
        assert!((mid_point.z() - 0.5).abs() < 1e-10);

        let length = line.length();
        assert!((length - (3.0_f64).sqrt()).abs() < 1e-10);
    }

    #[test]
    fn test_curve3d_downcasting() {
        let line: Box<dyn Curve3D> = curve_factory::create_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0)
        );

        // Any downcasting テスト
        let adapted_line = line.as_any().downcast_ref::<AdaptedLine>();
        assert!(adapted_line.is_some());
    }
}
