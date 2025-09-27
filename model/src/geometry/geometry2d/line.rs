use super::{point::Point, direction::Direction, infinit_line::InfiniteLine};
use crate::geometry_kind::CurveKind2D;
use crate::geometry_trait::Curve2D;
use crate::geometry_common::{IntersectionResult, IntersectionKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    base: InfiniteLine,
    start: Point,
    end: Point,
    length: f64,
}

impl Line {
    /// 始点と終点から Line を構築
    pub fn new(start: Point, end: Point) -> Self {
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let length = (dx * dx + dy * dy).sqrt();
        let direction = Direction::new(dx / length, dy / length);
        let base = InfiniteLine::new(start.clone(), direction);
        Self { base, start, end, length }
    }

    /// 始点を返す
    pub fn start(&self) -> &Point {
        &self.start
    }

    /// 終点を返す
    pub fn end(&self) -> &Point {
        &self.end
    }

    /// 中点を返す
    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.start.x + self.end.x) * 0.5,
            (self.start.y + self.end.y) * 0.5,
        )
    }

    /// 無限直線としての表現を返す
    pub fn to_infinite(&self) -> &InfiniteLine {
        &self.base
    }

    /// 他の InfiniteLine との交差判定（語義 + 点 + パラメータ + 誤差）
    pub fn intersection_with_infinite_line(&self, other: &InfiniteLine, epsilon: f64) -> IntersectionResult {
        let result = self.to_infinite().intersection_with_infinite_line(other, epsilon);

        let pts = result.points
            .into_iter()
            .filter(|pt| self.contains_point(pt, epsilon))
            .collect::<Vec<_>>();

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: vec![], // Line における t は必要なら後で追加可能
            tolerance_used: epsilon,
        }
    }

    /// 他の Line との交差判定（語義 + 点 + パラメータ + 誤差）
    pub fn intersection_with_line(&self, other: &Line, epsilon: f64) -> IntersectionResult<Point> {
        let ab = Point::new(self.end.x - self.start.x, self.end.y - self.start.y);
        let cd = Point::new(other.end.x - other.start.x, other.end.y - other.start.y);
        let det = ab.x * cd.y - ab.y * cd.x;

        if det.abs() < epsilon {
            // 平行または一致
            if self.contains_point(&other.start, epsilon) && self.contains_point(&other.end, epsilon) {
                return IntersectionResult::overlap(epsilon);
            } else {
                return IntersectionResult::none(epsilon);
            }
        }

        let dx = other.start.x - self.start.x;
        let dy = other.start.y - self.start.y;

        let t_self = (dx * cd.y - dy * cd.x) / det;
        let t_other = (dx * ab.y - dy * ab.x) / det;

        if t_self >= -epsilon && t_self <= 1.0 + epsilon && t_other >= -epsilon && t_other <= 1.0 + epsilon {
            let pt = self.evaluate(t_self);
            let kind = if t_self.abs() < epsilon || (1.0 - t_self).abs() < epsilon
                    || t_other.abs() < epsilon || (1.0 - t_other).abs() < epsilon {
                IntersectionKind::Tangent
            } else {
                IntersectionKind::Point
            };

            return IntersectionResult {
                kind,
                points: vec![pt],
                parameters: vec![t_self], // ✅ t_self を記録（将来的に t_other も保持可能）
                tolerance_used: epsilon,
            };
        }

        IntersectionResult::none(epsilon)
    }
}

impl Curve2D for Line {
    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Line
    }

    /// パラメータ t に対応する点を返す（t ∈ [0, 1]）
    fn evaluate(&self, t: f64) -> Point {
        self.base.evaluate(t * self.length)
    }

    /// 接線ベクトル（方向 × 長さ）
    fn derivative(&self) -> Point {
        Point::new(self.base.direction().x * self.length, self.base.direction().y * self.length)
    }

    /// 長さを返す
    fn length(&self) -> f64 {
        self.length
    }
}
