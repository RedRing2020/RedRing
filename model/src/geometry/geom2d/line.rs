use super::{point::Point2D, direction::Direction2D, geometry_kind::GeometryKind2D, intersect::Intersect2D};

#[derive(Debug, Clone, PartialEq)]
pub struct Line2D {
    origin: Point2D,
    direction: Direction2D,
    length: f64,
}

// 公開APIは必要最小限に限定
impl Line2D {
    pub fn new(origin: Point2D, direction: Direction2D, length: f64) -> Self {
        Self { origin, direction, length }
    }

    pub fn origin(&self) -> &Point2D {
        &self.origin
    }

    pub fn direction(&self) -> &Direction2D {
        &self.direction
    }

    pub fn length(&self) -> f64 {
        self.length
    }

    pub fn set_length(&mut self, new_length: f64) {
        self.length = new_length.max(0.0);
    }

    pub fn evaluate(&self, t: f64) -> Point2D {
        self.origin.add(self.direction.x * self.length * t, self.direction.y * self.length * t)
    }

    pub fn end_point(&self) -> Point2D {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point2D {
        self.evaluate(0.5)
    }

    pub fn intersects_line(&self, other: &Line2D, epsilon: f64) -> bool {
        self.intersection_with_line(other, epsilon).len() > 0
    }

    pub fn intersection_with_line(&self, other: &Line2D, epsilon: f64) -> Vec<Point2D> {
        let p1 = self.start;
        let p2 = self.end;
        let p3 = other.start;
        let p4 = other.end;

        let d1 = p2.sub(&p1);
        let d2 = p4.sub(&p3);

        let det = d1.x * d2.y - d1.y * d2.x;
        if det.abs() < epsilon {
            return vec![]; // 平行または一致
        }

        let dx = p3.x - p1.x;
        let dy = p3.y - p1.y;

        let t = (dx * d2.y - dy * d2.x) / det;
        let u = (dx * d1.y - dy * d1.x) / det;

        if t < -epsilon || t > 1.0 + epsilon || u < -epsilon || u > 1.0 + epsilon {
            return vec![]; // 線分範囲外
        }

        let ix = p1.x + t * d1.x;
        let iy = p1.y + t * d1.y;
        vec![Point2D::new(ix, iy)]
    }

    pub fn intersects_circle(&self, circle: &Circle2D, epsilon: f64) -> bool {
        self.intersection_with_circle(circle, epsilon).len() > 0
    }

    pub fn intersection_with_circle(&self, circle: &Circle2D, epsilon: f64) -> Vec<Point2D> {
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
                result.push(Point2D::new(ix, iy));
            }
        }

        result
    }
}

impl Intersect2D for Line2D {
    fn intersects_with(&self, other: &GeometryKind2D, epsilon: f64) -> bool {
        match other {
            GeometryKind2D::Line(line2) => self.intersects_line(line2, epsilon),
            GeometryKind2D::Circle(circle) => self.intersects_circle(circle, epsilon),
            // 他の形状は後続で追加
            _ => false,
        }
    }

    fn intersection_points(&self, other: &GeometryKind2D, epsilon: f64) -> Vec<Point2D> {
        match other {
            GeometryKind2D::Line(line2) => self.intersection_with_line(line2, epsilon),
            GeometryKind2D::Circle(circle) => self.intersection_with_circle(circle, epsilon),
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{point::Point2D, direction::Direction2D};

    #[test]
    fn test_evaluate() {
        let origin = Point2D::new(0.0, 0.0);
        let dir = Direction2D::new(1.0, 0.0);
        let line = Line2D::new(origin, dir, 10.0);
        let p = line.evaluate(0.5);
        assert_eq!(p, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_end_point() {
        let origin = Point2D::new(1.0, 2.0);
        let dir = Direction2D::new(0.0, 1.0);
        let line = Line2D::new(origin, dir, 3.0);
        let end = line.end_point();
        assert_eq!(end, Point2D::new(1.0, 5.0));
    }

    #[test]
    fn test_midpoint() {
        let origin = Point2D::new(0.0, 0.0);
        let dir = Direction2D::new(1.0, 0.0);
        let line = Line2D::new(origin, dir, 10.0);
        let mid = line.midpoint();
        assert_eq!(mid, Point2D::new(5.0, 0.0));
    }

    #[test]
    fn test_from_points() {
        let start = Point2D::new(2.0, 2.0);
        let end = Point2D::new(6.0, 2.0);
        let line = Line2D::from_points(start, end);
        assert_eq!(line.origin, start);
        assert_eq!(line.end_point(), end);
        assert_eq!(line.length, 4.0);
        assert_eq!(line.direction, Direction2D::new(1.0, 0.0));
    }

    #[test]
    fn test_line_intersection_at_center() {
        let a = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 2.0));
        let b = Line2D::new(Point2D::new(0.0, 2.0), Point2D::new(2.0, 0.0));
        let pts = a.intersection_with_line(&b, 1e-10);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].x - 1.0).abs() < 1e-10);
        assert!((pts[0].y - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_line_circle_two_points() {
        let line = Line2D::new(Point2D::new(-5.0, 0.0), Point2D::new(5.0, 0.0));
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0, Direction2D::new(0.0, 1.0));
        let pts = line.intersection_with_circle(&circle, 1e-10);
        assert_eq!(pts.len(), 2);
        assert!((pts[0].x + 3.0).abs() < 1e-10 || (pts[0].x - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_circle_tangent() {
        let line = Line2D::new(Point2D::new(-3.0, 3.0), Point2D::new(3.0, 3.0));
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 3.0, Direction2D::new(0.0, 1.0));
        let pts = line.intersection_with_circle(&circle, 1e-10);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].y - 3.0).abs() < 1e-10);
    }
}