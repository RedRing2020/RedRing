use super::point::Point2D;

#[derive(Debug, Clone, PartialEq)]
pub enum IntersectionKind2D {
    Point,
    Tangent,
    Overlap,
    None,
}

/// 交差結果（意味論 + 交点 + パラメータ + 使用した誤差許容値）
#[derive(Debug, Clone)]
pub struct IntersectionResult2D {
    pub kind: IntersectionKind2D,
    pub points: Vec<Point2D>,
    pub parameters: Vec<f64>,
    pub tolerance_used: f64,
}