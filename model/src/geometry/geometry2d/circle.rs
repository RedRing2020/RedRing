

use crate::geometry::geometry2d::{
    point::Point,
    vector::Vector,
    direction::Direction,
};


#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point,
    radius: f64,
    direction: Direction, // 回転方向（通常は正方向 = 反時計回り）
}

impl Circle {
    pub fn new(center: Point, radius: f64, direction: Direction) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
            direction,
        }
    }

    pub fn center(&self) -> &Point { &self.center }

    pub fn radius(&self) -> f64 { self.radius }

    pub fn direction(&self) -> &Direction {
        &self.direction
    }

    /// 円の回転方向を反転
    pub fn reverse_direction(&mut self) {
        self.direction = self.direction.reverse();
    }

    /// θ ∈ [0, 2π) における接線方向（右手系）
    pub fn tangent(&self, theta: f64) -> Vector {
        Vector::new(-theta.sin(), theta.cos())
    }

    /// θ ∈ [0, 2π) における法線方向（中心から外向き）
    pub fn normal(&self, theta: f64) -> Vector {
        Vector::new(theta.cos(), theta.sin())
    }

    /// 点が円周上にあるかどうか（誤差 ε を考慮）
    pub fn contains_point(&self, pt: &Point, epsilon: f64) -> bool {
        let dist_sq = (pt.x() - self.center.x()).powi(2) + (pt.y() - self.center.y()).powi(2);
        (dist_sq - self.radius.powi(2)).abs() < epsilon
    }
/*
    /// 無限直線との交差点を求める
    pub fn intersection_with_infinite_line(&self, line: &InfiniteLine, epsilon: f64) -> IntersectionResult<Point> {
        let dx = line.origin().x() - self.center.x();
        let dy = line.origin().y - self.center.y();

        let normal = line.direction().right_hand_normal();
        let dist = dx * normal.x() + dy * normal.y();

        if dist.abs() > self.radius + epsilon {
            return IntersectionResult::none(epsilon);
        }

        let d = dist.clamp(-self.radius, self.radius);
        let offset = Direction::new(normal.x() * d, normal.y() * d);
        let foot = self.center.add(offset.x(), offset.y());

        let h = (self.radius.powi(2) - d.powi(2)).sqrt();
        let dir = line.direction();
        let p1 = foot.add(dir.x() * h, dir.y() * h);
        let p2 = foot.add(-dir.x() * h, -dir.y() * h);

        let kind = if h < epsilon {
            IntersectionKind::Tangent
        } else {
            IntersectionKind::Point
        };

        IntersectionResult {
            kind,
            points: if h < epsilon { vec![foot] } else { vec![p1, p2] },
            parameters: vec![],
            tolerance_used: epsilon,
        }
    }

    /// 線分との交差点を求める
    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult<Point> {
        let pts = line.intersection_with_circle(self);
        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };
        IntersectionResult {
            kind,
            points: pts,
            parameters: vec![],
            tolerance_used: EPSILON,
        }
    }

    pub fn intersection_with_arc(&self, arc: &Arc, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| arc.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let theta = arc.angle_of(&pt);
                let t = (theta - arc.start_angle).rem_euclid(std::f64::consts::TAU) / arc.sweep_angle();
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (*a - *b).abs() < epsilon);

        let kind = match points.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points,
            parameters,
            tolerance_used: epsilon,
        }
    }

    pub fn intersection_with_ellipse(&self, ellipse: &Ellipse, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |theta| ellipse.evaluate(theta),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_hint(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (*a - *b).abs() < epsilon);

        let kind = match points.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points,
            parameters,
            tolerance_used: epsilon,
        }
    }
*/
}
/*
impl Curve2D for Circle {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::Circle
    }

    /// θ ∈ [0, 2π) における点を評価
    fn evaluate(&self, t: f64) -> Point {
        let dx = self.radius * t.cos();
        let dy = self.radius * t.sin();
        self.center.add(dx, dy)
    }

    fn derivative(&self, t: f64) -> Vector {
        self.tangent(t)
    }

    fn length(&self) -> f64 {
        std::f64::consts::TAU * self.radius()
    }

    fn parameter_hint(&self, pt: &Point) -> f64 {
        // 例えば中心からの角度などで推定
        let v: Vector = pt.sub(&self.center).to_vector();
        v.angle() / (2.0 * std::f64::consts::PI)
    }
}
*/
