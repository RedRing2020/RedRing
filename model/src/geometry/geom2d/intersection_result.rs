use super::point::Point2;

#[derive(Debug, Clone, PartialEq)]
pub enum IntersectionKind2 {
    Point,
    Tangent,
    Overlap,
    None,
}

/// 交差結果（意味論 + 交点 + パラメータ + 使用した誤差許容値）
#[derive(Debug, Clone)]
pub struct IntersectionResult2 {
    pub kind: IntersectionKind2,
    pub points: Vec<Point2>,
    pub parameters: Vec<f64>,
    pub tolerance_used: f64,
}