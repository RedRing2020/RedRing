use super::{point::Point2, direction::Direction2, geometry_kind::GeometryKind2, intersect::Intersect2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Line2 {
    origin: Point2,
    direction: Direction2,
    length: f64,
}

// 公開APIは必要最小限に限定
impl Line2 {
    pub fn new(origin: Point2, direction: Direction2, length: f64) -> Self {
        Self { origin, direction, length }
    }

    pub fn origin(&self) -> &Point2 {
        &self.origin
    }

    pub fn direction(&self) -> &Direction2 {
        &self.direction
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn set_length(&mut self, new_length: f64) {
        self.length = new_length.max(0.0);
    }

    pub fn evaluate(&self, t: f64) -> Point2 {
        self.origin.add(self.direction.x * self.length * t, self.direction.y * self.length * t)
    }

    pub fn end_point(&self) -> Point2 {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point2 {
        self.evaluate(0.5)
    }

    pub fn intersects_line(&self, other: &Line2, epsilon: f64) -> bool {
        self.intersection_with_line(other, epsilon).len() > 0
    }

    pub fn intersection_with_line(&self, other: &Line2) -> IntersectionResult2 {
        let ab = self.end.sub(&self.start);
        let cd = other.end.sub(&other.start);
        let det = ab.cross(&cd);

        if det.abs() < EPSILON {
            // 平行または一致
            if self.contains_point(&other.start) && self.contains_point(&other.end) {
                return IntersectionResult2 {
                    kind: IntersectionKind2::Overlap,
                    points: vec![],
                    parameters: vec![],
                    tolerance_used: EPSILON,
                };
            } else {
                return IntersectionResult2 {
                    kind: IntersectionKind2::None,
                    points: vec![],
                    parameters: vec![],
                    tolerance_used: EPSILON,
                };
            }
        }

        // 線分交差判定（2D線形代数）
        let t = (other.start.sub(&self.start)).cross(&cd) / det;
        let u = (other.start.sub(&self.start)).cross(&ab) / det;

        if t >= -EPSILON && t <= 1.0 + EPSILON && u >= -EPSILON && u <= 1.0 + EPSILON {
            let pt = Point2::new(
                self.start.x + t * ab.x,
                self.start.y + t * ab.y,
            );
            let kind = if t.abs() < EPSILON || (1.0 - t).abs() < EPSILON || u.abs() < EPSILON || (1.0 - u).abs() < EPSILON {
                IntersectionKind2::Tangent
            } else {
                IntersectionKind2::Point
            };
            return IntersectionResult2 {
                kind,
                points: vec![pt],
                parameters: vec![t],
                tolerance_used: EPSILON,
            };
        }

        IntersectionResult2 {
            kind: IntersectionKind2::None,
            points: vec![],
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn intersects_circle(&self, circle: &Circle2, epsilon: f64) -> bool {
        self.intersection_with_circle(circle, epsilon).len() > 0
    }

    pub fn intersection_with_circle(&self, circle: &Circle2, epsilon: f64) -> Vec<Point2> {
        let p1 = self.start;
        let p2 = self.end;
        let c = circle.center;
        let r = circle.radius;

        let d = p2.sub(&p1); // 線分方向ベクトル
        let f = p1.sub(&c);  // 円中心から線分始点へのベクトル

        let a = d.dot(&d);
        let b = 2.0 * f.dot(&d);
        let c_val = f.dot(&f) - r * r;

        let discriminant = b * b - 4.0 * a * c_val;
        if discriminant < -epsilon {
            return vec![]; // 交差なし
        }

        let mut result = vec![];
        let sqrt_disc = discriminant.max(0.0).sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        for &t in &[t1, t2] {
            if t >= -epsilon && t <= 1.0 + epsilon {
                let ix = p1.x + t * d.x;
                let iy = p1.y + t * d.y;
                result.push(Point2::new(ix, iy));
            }
        }

        result
    }
}

impl Intersect2D for Line2 {
    fn intersects_with(&self, other: &GeometryKind2, epsilon: f64) -> bool {
        match other {
            GeometryKind2::Line(line2) => self.intersects_line(line2, epsilon),
            GeometryKind2::Circle(circle) => self.intersects_circle(circle, epsilon),
            // 他の形状は後続で追加
            _ => false,
        }
    }

    fn intersection_points(&self, other: &GeometryKind2, epsilon: f64) -> Vec<Point2> {
        match other {
            GeometryKind2::Line(line2) => self.intersection_with_line(line2, epsilon),
            GeometryKind2::Circle(circle) => self.intersection_with_circle(circle, epsilon),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2, direction::Direction2};

    #[test]
    fn test_evaluate() {
        let origin = Point2::new(0.0, 0.0);
        let dir = Direction2::new(1.0, 0.0);
        let line = Line2::new(origin, dir, 10.0);
        let p = line.evaluate(0.5);
        assert_eq!(p, Point2::new(5.0, 0.0));
    }

    #[test]
    fn test_end_point() {
        let origin = Point2::new(1.0, 2.0);
        let dir = Direction2::new(0.0, 1.0);
        let line = Line2::new(origin, dir, 3.0);
        let end = line.end_point();
        assert_eq!(end, Point2::new(1.0, 5.0));
    }

    #[test]
    fn test_midpoint() {
        let origin = Point2::new(0.0, 0.0);
        let dir = Direction2::new(1.0, 0.0);
        let line = Line2::new(origin, dir, 10.0);
        let mid = line.midpoint();
        assert_eq!(mid, Point2::new(5.0, 0.0));
    }

    #[test]
    fn test_from_points() {
        let start = Point2::new(2.0, 2.0);
        let end = Point2::new(6.0, 2.0);
        let line = Line2::from_points(start, end);
        assert_eq!(line.origin, start);
        assert_eq!(line.end_point(), end);
        assert_eq!(line.length, 4.0);
        assert_eq!(line.direction, Direction2::new(1.0, 0.0));
    }

    #[test]
    fn test_line_intersection_at_center() {
        let a = Line2::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0));
        let b = Line2::new(Point2::new(0.0, 2.0), Point2::new(2.0, 0.0));
        let pts = a.intersection_with_line(&b, 1e-10);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].x - 1.0).abs() < 1e-10);
        assert!((pts[0].y - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_line_circle_two_points() {
        let line = Line2::new(Point2::new(-5.0, 0.0), Point2::new(5.0, 0.0));
        let circle = Circle2::new(Point2::new(0.0, 0.0), 3.0, Direction2::new(0.0, 1.0));
        let pts = line.intersection_with_circle(&circle, 1e-10);
        assert_eq!(pts.len(), 2);
        assert!((pts[0].x + 3.0).abs() < 1e-10 || (pts[0].x - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_circle_tangent() {
        let line = Line2::new(Point2::new(-3.0, 3.0), Point2::new(3.0, 3.0));
        let circle = Circle2::new(Point2::new(0.0, 0.0), 3.0, Direction2::new(0.0, 1.0));
        let pts = line.intersection_with_circle(&circle, 1e-10);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].y - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_line_intersection_point() {
        let a = Line2::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0));
        let b = Line2::new(Point2::new(0.0, 2.0), Point2::new(2.0, 0.0));
        let result = a.intersection_with_line(&b);
        assert_eq!(result.kind, IntersectionKind2::Point);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn test_line_line_overlap() {
        let a = Line2::new(Point2::new(0.0, 0.0), Point2::new(2.0, 2.0));
        let b = Line2::new(Point2::new(1.0, 1.0), Point2::new(3.0, 3.0));
        let result = a.intersection_with_line(&b);
        assert_eq!(result.kind, IntersectionKind2::Overlap);
    }
}