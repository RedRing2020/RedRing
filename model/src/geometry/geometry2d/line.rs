
use super::{point::Point, direction::Direction, infinite_line::InfiniteLine};

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
        let dx = end.x() - start.x();
        let dy = end.y() - start.y();
        let length = (dx * dx + dy * dy).sqrt();
        let direction = Direction::new(dx / length, dy / length);
        let base = InfiniteLine::new(start.clone(), direction);
        Self { base, start, end, length }
    }

    /// 始点を返す
    pub fn start(&self) -> Point { self.start }

    /// 終点を返す
    pub fn end(&self) -> Point { self.end }

    /// 長さを返す
    pub fn length(&self) -> f64 { self.length }

    /// 中点を返す
    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.start.x() + self.end.x()) * 0.5,
            (self.start.y() + self.end.y()) * 0.5,
        )
    }

    /// 無限直線としての表現を返す
    pub fn to_infinite(&self) -> &InfiniteLine {
        &self.base
    }

    /// 点が線分上にあるかどうか（誤差 ε を考慮）
    pub fn contains_point(&self, pt: &Point, epsilon: f64) -> bool { false }

    /// 線分上の最近点を返す
    fn closest_point(&self, pt: &Point) -> Point {
        let dx = self.end.x() - self.start.x();
        let dy = self.end.y() - self.start.y();
        let len_sq = dx * dx + dy * dy;
        if len_sq == 0.0 {
            return self.start;
        }
        let t = ((pt.x() - self.start.x()) * dx + (pt.y() - self.start.y()) * dy) / len_sq;
        let t_clamped = t.clamp(0.0, 1.0);
        Point::new(
            self.start.x() + t_clamped * dx,
            self.start.y() + t_clamped * dy,
        )
    }

    /// 点ptから線分への距離
    pub fn distance_to_point(&self, pt: &Point) -> f64 {
        let closest = self.closest_point(pt);
        let dx = pt.x() - closest.x();
        let dy = pt.y() - closest.y();
        (dx * dx + dy * dy).sqrt()
    }
/*
    /// 他の InfiniteLine との交差判定（語義 + 点 + パラメータ + 誤差）
    pub fn intersection_with_infinite_line(&self, other: &InfiniteLine, epsilon: f64) -> IntersectionResult<Point> {
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

    /// 点ptから線分Lineへの距離
    pub fn distance_to_point(&self, pt: &Point) -> f64 {
        // 線分のベクトル
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;

        // 始点からptへのベクトル
        let px = pt.x() - self.start.x();
        let py = pt.y() - self.start.y();

        // 線分上の最近点のパラメータt（0 <= t <= 1）
        let len_sq = dx * dx + dy * dy;
        let t = if len_sq == 0.0 {
            0.0
        } else {
            ((px * dx + py * dy) / len_sq).clamp(0.0, 1.0)
        };

        // 最近点座標
        let closest_x = self.start.x() + t * dx;
        let closest_y = self.start.y() + t * dy;

        // ptと最近点の距離
        let dist_x = pt.x() - closest_x();
        let dist_y = pt.y() - closest_y();
        (dist_x * dist_x + dist_y * dist_y).sqrt()
    }
*/
}
/*
impl Curve2D for Line {
    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Line
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn evaluate(&self, t: f64) -> Point {
        self.base.evaluate(t * self.length)
    }

    fn derivative(&self, _t: f64) -> Vector {
        Vector::new(self.base.direction().x() * self.length, self.base.direction().y() * self.length)
    }

    fn length(&self) -> f64 {
        self.length
    }
}
*/
