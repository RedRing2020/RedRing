//! 幾何学的交差結果の共通定義（2D/3D 両対応）
//!
//! Curve2D / Curve3D / Surface などの交差判定に使用される。
//! 誤差判定は `analysis::consts` に準拠。

/// 交差の意味論的分類
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

/// 交差結果（意味論 + 交点 + パラメータ + 使用した誤差許容値）
///
/// ジェネリクス P は Point2D / Point3D に対応可能
#[derive(Debug, Clone)]
pub struct IntersectionResult<P> {
    pub kind: IntersectionKind,
    pub points: Vec<P>,           // Point2D or Point3D
    pub parameters: Vec<f64>,     // Curve 上の t 値（語義整合済み）
    pub tolerance_used: f64,
}

impl<P> IntersectionResult<P> {
    /// 交差なしの結果を構築
    pub fn none(epsilon: f64) -> Self {
        Self {
            kind: IntersectionKind::None,
            points: vec![],
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    /// 通常交差（1点）の結果を構築
    pub fn point(pt: P, t: f64, epsilon: f64) -> Self {
        Self {
            kind: IntersectionKind::Point,
            points: vec![pt],
            parameters: vec![t],
            tolerance_used: epsilon,
        }
    }

    /// 複数交差点の結果を構築
    pub fn points(pts: Vec<P>, ts: Vec<f64>, epsilon: f64) -> Self {
        Self {
            kind: IntersectionKind::Point,
            points: pts,
            parameters: ts,
            tolerance_used: epsilon,
        }
    }

    /// 接線交差の結果を構築
    pub fn tangent(pt: P, t: f64, epsilon: f64) -> Self {
        Self {
            kind: IntersectionKind::Tangent,
            points: vec![pt],
            parameters: vec![t],
            tolerance_used: epsilon,
        }
    }

    /// 重なり（交差区間）の結果を構築（点群なし）
    pub fn overlap(epsilon: f64) -> Self {
        Self {
            kind: IntersectionKind::Overlap,
            points: vec![],
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    pub fn is_none(&self) -> bool {
        self.kind == IntersectionKind::None
    }

    pub fn is_tangent(&self) -> bool {
        self.kind == IntersectionKind::Tangent
    }

    pub fn is_overlap(&self) -> bool {
        self.kind == IntersectionKind::Overlap
    }
}
