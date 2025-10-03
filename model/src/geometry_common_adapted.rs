/// geometry_common/intersection.rs の geo_core 統合版
/// 
/// IntersectionResult<P> をgeo_coreのToleranceContextと統合し、
/// 既存のジェネリック設計を保持しつつ数値的堅牢性を向上

use geo_core::{ToleranceContext, TolerantEq};

/// 交差の意味論的分類（既存設計を保持）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionKind {
    /// 通常交差（有限個の交点）
    Point,
    /// 接触（接線的交差）
    Tangent,
    /// 重なり（区間的交差）
    Overlap,
    /// 交差なし
    None,
}

/// 交差結果（geo_core ToleranceContext統合版）
///
/// ジェネリクス P は Point2D / Point3D に対応、
/// geo_core::Point3D と model::Point3D の両方で使用可能
#[derive(Debug, Clone)]
pub struct IntersectionResult<P> {
    pub kind: IntersectionKind,
    pub points: Vec<P>,
    pub parameters: Vec<f64>,
    /// geo_core の ToleranceContext を使用
    pub tolerance_context: ToleranceContext,
}

impl<P> IntersectionResult<P> {
    /// 交差なしの結果を構築（geo_core ToleranceContext使用）
    pub fn none(tolerance: ToleranceContext) -> Self {
        Self {
            kind: IntersectionKind::None,
            points: vec![],
            parameters: vec![],
            tolerance_context: tolerance,
        }
    }

    /// 通常交差（1点）の結果を構築
    pub fn point(pt: P, t: f64, tolerance: ToleranceContext) -> Self {
        Self {
            kind: IntersectionKind::Point,
            points: vec![pt],
            parameters: vec![t],
            tolerance_context: tolerance,
        }
    }

    /// 接触交差の結果を構築
    pub fn tangent(pt: P, t: f64, tolerance: ToleranceContext) -> Self {
        Self {
            kind: IntersectionKind::Tangent,
            points: vec![pt],
            parameters: vec![t],
            tolerance_context: tolerance,
        }
    }

    /// 重なり交差（複数点）の結果を構築
    pub fn overlap(points: Vec<P>, parameters: Vec<f64>, tolerance: ToleranceContext) -> Self {
        Self {
            kind: IntersectionKind::Overlap,
            points,
            parameters,
            tolerance_context: tolerance,
        }
    }

    /// 複数点交差の結果を構築
    pub fn multiple_points(points: Vec<P>, parameters: Vec<f64>, tolerance: ToleranceContext) -> Self {
        Self {
            kind: IntersectionKind::Point,
            points,
            parameters,
            tolerance_context: tolerance,
        }
    }

    /// 交差があるかチェック
    pub fn has_intersection(&self) -> bool {
        self.kind != IntersectionKind::None
    }

    /// 交点数を取得
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// 使用された線形許容誤差を取得（後方互換性）
    pub fn tolerance_used(&self) -> f64 {
        self.tolerance_context.linear
    }

    /// 使用された角度許容誤差を取得
    pub fn angular_tolerance(&self) -> f64 {
        self.tolerance_context.angular
    }

    /// 使用されたパラメトリック許容誤差を取得
    pub fn parametric_tolerance(&self) -> f64 {
        self.tolerance_context.parametric
    }
}

/// トレラント比較を使用した交点判定ユーティリティ
pub mod intersection_utils {
    use super::*;
    use crate::geometry_adapter::{Point3D, Vector3D};

    /// 2つの点が許容誤差内で一致するかチェック
    pub fn points_are_coincident(
        p1: &Point3D, 
        p2: &Point3D, 
        tolerance: &ToleranceContext
    ) -> bool {
        let geo_p1 = p1.as_geo_core();
        let geo_p2 = p2.as_geo_core();
        geo_p1.tolerant_eq(geo_p2, tolerance)
    }

    /// パラメータ値が許容誤差内で一致するかチェック
    pub fn parameters_are_coincident(
        t1: f64,
        t2: f64,
        tolerance: &ToleranceContext
    ) -> bool {
        (t1 - t2).abs() <= tolerance.parametric
    }

    /// ベクトルが許容誤差内で平行かチェック
    pub fn vectors_are_parallel(
        v1: &Vector3D,
        v2: &Vector3D,
        tolerance: &ToleranceContext
    ) -> bool {
        let geo_v1 = v1.as_geo_core();
        let geo_v2 = v2.as_geo_core();
        
        let cross = geo_v1.cross(geo_v2);
        let cross_magnitude = cross.magnitude();
        
        cross_magnitude.value() <= tolerance.linear
    }

    /// ベクトルが許容誤差内で垂直かチェック
    pub fn vectors_are_perpendicular(
        v1: &Vector3D,
        v2: &Vector3D,
        tolerance: &ToleranceContext
    ) -> bool {
        let geo_v1 = v1.as_geo_core();
        let geo_v2 = v2.as_geo_core();
        
        let dot = geo_v1.dot(geo_v2);
        dot.value().abs() <= tolerance.linear
    }
}

/// 交差解析のコンテキスト管理
pub struct IntersectionContext {
    tolerance: ToleranceContext,
    max_iterations: usize,
    convergence_threshold: f64,
}

impl IntersectionContext {
    /// 標準設定で作成
    pub fn standard() -> Self {
        Self {
            tolerance: ToleranceContext::standard(),
            max_iterations: 100,
            convergence_threshold: 1e-12,
        }
    }

    /// カスタム許容誤差で作成
    pub fn with_tolerance(tolerance: ToleranceContext) -> Self {
        Self {
            tolerance,
            max_iterations: 100,
            convergence_threshold: 1e-12,
        }
    }

    /// 高精度設定で作成
    pub fn high_precision() -> Self {
        Self {
            tolerance: ToleranceContext {
                linear: 1e-9,
                angular: 1e-12,
                parametric: 1e-15,
                curvature: 1e-6,
                area: 1e-18,
                volume: 1e-27,
            },
            max_iterations: 1000,
            convergence_threshold: 1e-15,
        }
    }

    pub fn tolerance(&self) -> &ToleranceContext {
        &self.tolerance
    }

    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    pub fn convergence_threshold(&self) -> f64 {
        self.convergence_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_adapter::Point3D;

    #[test]
    fn test_intersection_result_creation() {
        let tolerance = ToleranceContext::standard();
        let pt = Point3D::new(1.0, 2.0, 3.0);
        
        let result = IntersectionResult::point(pt, 0.5, tolerance.clone());
        
        assert_eq!(result.kind, IntersectionKind::Point);
        assert_eq!(result.points.len(), 1);
        assert_eq!(result.parameters.len(), 1);
        assert_eq!(result.parameters[0], 0.5);
        assert!(result.has_intersection());
    }

    #[test]
    fn test_intersection_context() {
        let context = IntersectionContext::standard();
        assert_eq!(context.max_iterations(), 100);
        
        let high_prec = IntersectionContext::high_precision();
        assert_eq!(high_prec.max_iterations(), 1000);
        assert!(high_prec.tolerance().linear < context.tolerance().linear);
    }

    #[test]
    fn test_tolerance_compatibility() {
        let tolerance = ToleranceContext::standard();
        let result = IntersectionResult::<Point3D>::none(tolerance.clone());
        
        // 後方互換性テスト
        assert_eq!(result.tolerance_used(), tolerance.linear);
        assert_eq!(result.angular_tolerance(), tolerance.angular);
        assert_eq!(result.parametric_tolerance(), tolerance.parametric);
    }
}