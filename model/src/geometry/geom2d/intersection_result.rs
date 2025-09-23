use super::point::Point2D;

/// 交差の意味論を表すタグ
#[derive(Debug, Clone, PartialEq)]
pub enum IntersectionKind2D {
    Point,
    Tangent,
    Overlap,
    None,
}

/// 交差結果（意味論 + 交点 + パラメータ）
#[derive(Debug, Clone)]
pub struct IntersectionResult2D {
    pub kind: IntersectionKind2D,
    pub points: Vec<Point2D>,
    pub parameters: Vec<f64>, // NurbsCurve2D などで使用
}