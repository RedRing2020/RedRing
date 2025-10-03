use crate::geometry::geometry2d::{
    point::Point,
    ellipse::Ellipse,
};

use analysis::EPSILON;

#[derive(Debug, Clone, PartialEq)]
pub struct EllipseArc {
    ellipse: Ellipse,
    start_angle: f64, // ラジアン [0, 2π)
    end_angle: f64,   // ラジアン [0, 2π)
}

impl EllipseArc {
    pub fn new(ellipse: Ellipse, start_angle: f64, end_angle: f64) -> Self {
        Self { ellipse, start_angle, end_angle }
    }

    pub fn ellipse(&self) -> &Ellipse { &self.ellipse }

    pub fn start_angle(&self) -> &f64 { &self.start_angle }

    pub fn end_angle(&self) -> &f64 { &self.end_angle }


    pub fn sweep_angle(&self) -> f64 {
        let raw = self.end_angle - self.start_angle;
        if raw >= 0.0 { raw } else { raw + std::f64::consts::TAU }
    }

    pub fn evaluate(&self, t: f64) -> Point {
        let angle = self.start_angle + t * self.sweep_angle();
        self.ellipse.evaluate(angle)
    }

    pub fn start_point(&self) -> Point {
        self.evaluate(0.0)
    }

    pub fn end_point(&self) -> Point {
        self.evaluate(1.0)
    }

    pub fn midpoint(&self) -> Point {
        self.evaluate(0.5)
    }

    /// 楕円弧の角度範囲に含まれるか（方向付き）
    pub fn contains_angle(&self, theta: f64) -> bool {
        let start = self.start_angle;
        let end = self.end_angle;

        let sweep = if end >= start {
            end - start
        } else {
            end + std::f64::consts::TAU - start
        };

        let rel = if theta >= start {
            theta - start
        } else {
            theta + std::f64::consts::TAU - start
        };

        rel >= -EPSILON && rel <= sweep + EPSILON
    }
/*
    pub fn intersection_with_infinite_line(&self, line: &InfiniteLine, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| self.evaluate(t),
            line,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

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

    /// 半無限直線との交差点を求める
    pub fn intersection_with_ray(&self, ray: &Ray) -> IntersectionResult {
        let candidates = sample_intersections(
            |theta| self.ellipse.evaluate(theta),
            &ray.to_line(),
            360,
            EPSILON,
        );

        let mut pts = candidates
            .into_iter()
            .filter(|pt| {
                let theta = self.ellipse.angle_of(pt);
                self.contains_angle(theta) && ray.is_forward(pt)
            })
            .collect::<Vec<_>>();

        pts.dedup_by(|a, b| a.distance_to(b) < EPSILON);

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

    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult {
        let candidates = sample_intersections(
            |theta| self.ellipse.evaluate(theta),
            line,
            360,
            EPSILON,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            let theta = self.ellipse.angle_of(&pt);
            if self.contains_angle(theta) {
                let t = (theta - self.start_angle).rem_euclid(std::f64::consts::TAU) / self.sweep_angle();
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < EPSILON);
        parameters.dedup_by(|a, b| (a - b).abs() < EPSILON);

        let kind = match points.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points,
            parameters,
            tolerance_used: EPSILON,
        }
    }

    pub fn intersection_with_circle(&self, circle: &Circle, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| circle.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

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
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

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
            |t| ellipse.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

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

    pub fn intersection_with_ellipse_arc(&self, other: &EllipseArc, epsilon: f64) -> IntersectionResult<Point> {
        let candidates = sample_intersections(
            |t| other.evaluate(t),
            self,
            360,
            epsilon,
        );

        let mut points = vec![];
        let mut parameters = vec![];

        for pt in candidates {
            if self.contains_point(&pt, epsilon) {
                let t = self.parameter_of(&pt);
                points.push(pt);
                parameters.push(t);
            }
        }

        points.dedup_by(|a, b| a.distance_to(b) < epsilon);
        parameters.dedup_by(|a, b| (a - b).abs() < epsilon);

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
impl Curve2D for EllipseArc {
    fn kind(&self) -> CurveKind2D {
        CurveKind2D::EllipseArc
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn evaluate(&self, t: f64) -> Point {
        let angle = self.start_angle + t * self.sweep_angle();
        self.ellipse.evaluate(angle)
    }

    fn derivative(&self, t: f64) -> Vector {
        let angle = self.start_angle + t * self.sweep_angle();
        self.ellipse.tangent(angle)
    }

    fn length(&self) -> f64 {
        newton_arc_length(
            |theta| self.ellipse.evaluate(theta),
            self.start_angle,
            self.end_angle,
            360,
        )
    }
}
*/
