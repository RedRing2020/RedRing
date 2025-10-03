/// 階層統合設計によるLine統合実装例
/// 
/// geo_coreのLineSegment3Dを数値計算基盤として内包し、
/// modelのCAD概念（無限直線、トリミング）を保持する設計

use std::any::Any;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_trait::curve3d::Curve3D;
use super::{Vector3D, Point3D, Direction3D};
use geo_core::{
    LineSegment3D as GeoLineSegment3D,
    ToleranceContext,
    Scalar
};

/// 無限直線の情報
#[derive(Debug, Clone)]
pub struct InfiniteLineInfo {
    origin: Point3D,
    direction: Direction3D,
}

impl InfiniteLineInfo {
    pub fn new(origin: Point3D, direction: Direction3D) -> Self {
        Self { origin, direction }
    }

    pub fn origin(&self) -> &Point3D { &self.origin }
    pub fn direction(&self) -> &Direction3D { &self.direction }

    /// 無限直線上の点を取得（パラメータ t ∈ ℝ）
    pub fn at(&self, t: f64) -> Point3D {
        let direction_vec = self.direction.to_vector();
        let offset = direction_vec.scale(t);
        Point3D::new(
            self.origin.x() + offset.x(),
            self.origin.y() + offset.y(),
            self.origin.z() + offset.z()
        )
    }
}

/// トリミング情報
#[derive(Debug, Clone)]
pub struct TrimmingInfo {
    is_trimmed: bool,
    parameter_start: f64,  // 無限直線上でのstart位置
    parameter_end: f64,    // 無限直線上でのend位置
    infinite_line: Option<InfiniteLineInfo>,
}

impl TrimmingInfo {
    pub fn new_untrimmed() -> Self {
        Self {
            is_trimmed: false,
            parameter_start: 0.0,
            parameter_end: 1.0,
            infinite_line: None,
        }
    }

    pub fn new_trimmed(
        parameter_start: f64,
        parameter_end: f64,
        infinite_line: InfiniteLineInfo
    ) -> Self {
        Self {
            is_trimmed: true,
            parameter_start,
            parameter_end,
            infinite_line: Some(infinite_line),
        }
    }

    pub fn is_trimmed(&self) -> bool { self.is_trimmed }
    pub fn parameter_range(&self) -> (f64, f64) { (self.parameter_start, self.parameter_end) }
    pub fn infinite_line(&self) -> Option<&InfiniteLineInfo> { self.infinite_line.as_ref() }
}

/// 統合されたLine構造体
/// geo_coreの数値基盤 + modelのCAD概念
#[derive(Debug, Clone)]
pub struct IntegratedLine {
    /// 数値計算基盤（geo_core）
    core_segment: GeoLineSegment3D,
    /// CAD幾何情報
    trimming_info: TrimmingInfo,
    /// 許容誤差コンテキスト
    tolerance: ToleranceContext,
}

impl IntegratedLine {
    /// 2点から線分を作成（シンプルな線分）
    pub fn from_points(start: Point3D, end: Point3D) -> Self {
        let geo_start = start.as_geo_core().clone();
        let geo_end = end.as_geo_core().clone();
        
        Self {
            core_segment: GeoLineSegment3D::new(geo_start, geo_end),
            trimming_info: TrimmingInfo::new_untrimmed(),
            tolerance: ToleranceContext::standard(),
        }
    }

    /// 無限直線からトリミングして作成
    pub fn from_infinite_line_trimmed(
        infinite_line: InfiniteLineInfo,
        parameter_start: f64,
        parameter_end: f64
    ) -> Self {
        let start = infinite_line.at(parameter_start);
        let end = infinite_line.at(parameter_end);
        
        let geo_start = start.as_geo_core().clone();
        let geo_end = end.as_geo_core().clone();
        
        Self {
            core_segment: GeoLineSegment3D::new(geo_start, geo_end),
            trimming_info: TrimmingInfo::new_trimmed(
                parameter_start, 
                parameter_end, 
                infinite_line
            ),
            tolerance: ToleranceContext::standard(),
        }
    }

    /// 既存のmodel::Lineから変換
    pub fn from_model_line(
        origin: Point3D,
        direction: Direction3D,
        start: Point3D,
        end: Point3D
    ) -> Self {
        let infinite_line = InfiniteLineInfo::new(origin, direction);
        
        // 無限直線上でのstart/endパラメータを計算
        let start_param = Self::calculate_parameter_on_infinite_line(&infinite_line, &start);
        let end_param = Self::calculate_parameter_on_infinite_line(&infinite_line, &end);
        
        Self::from_infinite_line_trimmed(infinite_line, start_param, end_param)
    }

    /// 無限直線上での点のパラメータを計算
    fn calculate_parameter_on_infinite_line(
        infinite_line: &InfiniteLineInfo,
        point: &Point3D
    ) -> f64 {
        let origin_to_point = Vector3D::new(
            point.x() - infinite_line.origin().x(),
            point.y() - infinite_line.origin().y(),
            point.z() - infinite_line.origin().z()
        );
        let direction_vec = infinite_line.direction().to_vector();
        
        // 方向ベクトルとの内積でパラメータを求める
        origin_to_point.dot(&direction_vec) / direction_vec.dot(&direction_vec)
    }

    // アクセサメソッド
    pub fn start(&self) -> Point3D {
        Point3D::from_geo_core(self.core_segment.start().clone())
    }

    pub fn end(&self) -> Point3D {
        Point3D::from_geo_core(self.core_segment.end().clone())
    }

    pub fn direction(&self) -> Vector3D {
        Vector3D::from_geo_core(self.core_segment.direction())
    }

    pub fn trimming_info(&self) -> &TrimmingInfo {
        &self.trimming_info
    }

    pub fn is_aligned(&self) -> bool {
        if let Some(infinite_line) = self.trimming_info.infinite_line() {
            let segment_direction = self.direction().normalize();
            let infinite_direction = infinite_line.direction().to_vector().normalize();
            
            // 許容誤差内での方向一致判定
            let dot_product = segment_direction.dot(&infinite_direction).abs();
            (dot_product - 1.0).abs() < self.tolerance.angular
        } else {
            true // トリミング情報がない場合は整列していると見なす
        }
    }

    /// geo_coreの高精度計算を利用
    pub fn distance_to_point(&self, point: &Point3D) -> f64 {
        let geo_point = point.as_geo_core();
        self.core_segment.distance_to_point(geo_point).value()
    }

    pub fn closest_parameter(&self, point: &Point3D) -> f64 {
        let geo_point = point.as_geo_core();
        self.core_segment.closest_parameter(geo_point, &self.tolerance).value()
    }
}

impl Curve3D for IntegratedLine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind3D {
        CurveKind3D::Line
    }

    fn evaluate(&self, t: f64) -> Point3D {
        let geo_point = self.core_segment.evaluate(Scalar::new(t));
        Point3D::from_geo_core(geo_point)
    }

    fn derivative(&self, _t: f64) -> Vector3D {
        Vector3D::from_geo_core(self.core_segment.derivative(Scalar::new(_t)))
    }

    fn length(&self) -> f64 {
        self.core_segment.length().value()
    }

    fn parameter_hint(&self, pt: &Point3D) -> f64 {
        self.closest_parameter(pt)
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }
}

/// ファクトリー関数
pub mod integrated_line_factory {
    use super::*;

    pub fn create_line_segment(start: Point3D, end: Point3D) -> Box<dyn Curve3D> {
        Box::new(IntegratedLine::from_points(start, end))
    }

    pub fn create_trimmed_line(
        origin: Point3D,
        direction: Direction3D,
        start: Point3D,
        end: Point3D
    ) -> Box<dyn Curve3D> {
        Box::new(IntegratedLine::from_model_line(origin, direction, start, end))
    }

    pub fn create_infinite_line_segment(
        infinite_line: InfiniteLineInfo,
        parameter_start: f64,
        parameter_end: f64
    ) -> Box<dyn Curve3D> {
        Box::new(IntegratedLine::from_infinite_line_trimmed(
            infinite_line, parameter_start, parameter_end
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_line_basic() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(1.0, 0.0, 0.0);
        let line = IntegratedLine::from_points(start, end);

        assert_eq!(line.kind(), CurveKind3D::Line);
        assert!((line.length() - 1.0).abs() < 1e-10);
        
        let mid_point = line.evaluate(0.5);
        assert!((mid_point.x() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_trimmed_line() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::new(1.0, 0.0, 0.0).unwrap();
        let start = Point3D::new(2.0, 0.0, 0.0);
        let end = Point3D::new(5.0, 0.0, 0.0);

        let line = IntegratedLine::from_model_line(origin, direction, start, end);
        
        assert!(line.trimming_info().is_trimmed());
        assert!(line.is_aligned());
        
        let (param_start, param_end) = line.trimming_info().parameter_range();
        assert!((param_start - 2.0).abs() < 1e-10);
        assert!((param_end - 5.0).abs() < 1e-10);
    }
}