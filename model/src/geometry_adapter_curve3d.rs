/// geometry_adapter.rs の Curve3D trait 統合拡張
///
/// 既存のCurve3D trait設計を保持しつつ、geo_primitivesの数値基盤を活用

use std::any::Any;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use super::{Vector3D, Point3D};
use geo_foundation::{
    ToleranceContext,
    Scalar
};
/// geometry_adapter.rs の Curve3D trait 統合拡張
///
/// 既存のCurve3D trait設計を保持しつつ、geo_primitivesの数値基盤を活用

use std::any::Any;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use super::{Vector3D, Point3D};
use geo_foundation::{
    ToleranceContext,
    Scalar
};
// TODO: geo_primitives LineSegment3D への移行は今後実装予定

/// TODO: 仮実装 - 将来geo_primitives LineSegment3D に置き換え予定
pub struct MockLineSegment3D {
    start: Point3D,
    end: Point3D,
}

impl MockLineSegment3D {
    pub fn new(start: Point3D, end: Point3D) -> Self {
        Self { start, end }
    }

    pub fn evaluate(&self, t: f64) -> Point3D {
        let x = self.start.x() + t * (self.end.x() - self.start.x());
        let y = self.start.y() + t * (self.end.y() - self.start.y());
        let z = self.start.z() + t * (self.end.z() - self.start.z());
        Point3D::new(x, y, z)
    }

    pub fn direction(&self) -> Vector3D {
        Vector3D::new(
            self.end.x() - self.start.x(),
            self.end.y() - self.start.y(),
            self.end.z() - self.start.z(),
        )
    }

    pub fn length(&self) -> f64 {
        self.direction().norm()
    }

    pub fn closest_parameter(&self, _pt: &Point3D, _tolerance: &ToleranceContext) -> f64 {
        0.5 // 仮実装
    }
}

/// geo_primitives LineSegment3D のアダプター実装（仮実装）
pub struct AdaptedLine {
    inner: MockLineSegment3D,
    tolerance: ToleranceContext,
}

impl AdaptedLine {
    pub fn new(start: Point3D, end: Point3D) -> Self {
        Self {
            inner: MockLineSegment3D::new(start, end),
            tolerance: ToleranceContext::standard(),
        }
    }
}

/// geo_primitives LineSegment3D のアダプター実装
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
        self.inner.evaluate(t)
    }

    fn derivative(&self, _t: f64) -> Vector3D {
        // 直線の微分は定ベクトル
        self.inner.direction()
    }

    fn length(&self) -> f64 {
        self.inner.length()
    }

    fn parameter_hint(&self, pt: &Point3D) -> f64 {
        // 最近点のパラメータを計算（仮実装）
        self.inner.closest_parameter(pt, &self.tolerance)
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// ファクトリー関数群
pub mod curve_factory {
    use super::*;

    pub fn create_line(start: Point3D, end: Point3D) -> Box<dyn Curve3D> {
        Box::new(AdaptedLine::new(start, end))
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
