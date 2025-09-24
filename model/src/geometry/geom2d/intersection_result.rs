use super::point::Point;

#[derive(Debug, Clone, PartialEq)]
pub enum IntersectionKind {
    Point,
    Tangent,
    Overlap,
    None,
}

/// 交差結果（意味論 + 交点 + パラメータ + 使用した誤差許容値）
#[derive(Debug, Clone)]
pub struct IntersectionResult {
    pub kind: IntersectionKind,
    pub points: Vec<Point>,
    pub parameters: Vec<f64>,
    pub tolerance_used: f64,
}