use super::point::Point2D;

use crate::model::analysis::solver::newton_solve;
use crate::model::analysis::consts::{EPSILON};

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve2D {
    degree: usize,
    control_points: Vec<Point2D>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    domain: (f64, f64),

    is_rational: bool, // 重み付きかどうか
    is_uniform: bool,  // ノットが一様かどうか
}

impl NurbsCurve2D {
    pub fn new(degree: usize, control_points: Vec<Point2D>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        assert_eq!(control_points.len(), weights.len(), "制御点と重みの数が一致しません");
        assert!(knots.len() >= control_points.len() + degree + 1, "ノットベクトルが不足しています");

        let domain = (
            knots[degree],
            knots[knots.len() - degree - 1],
        );

        let is_rational = weights.iter().any(|w| (w - 1.0).abs() > 1e-10);
        let uniform_step = knots[1] - knots[0];
        let is_uniform = knots.windows(2).all(|w| (w[1] - w[0] - uniform_step).abs() < 1e-10);

        Self {
            degree,
            control_points,
            weights,
            knots,
            domain,
            is_rational,
            is_uniform,
        }
    }

    pub fn is_rational(&self) -> bool {
        self.is_rational
    }

    pub fn is_uniform(&self) -> bool {
        self.is_uniform
    }

    pub fn domain(&self) -> (f64, f64) {
        self.domain
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn control_points(&self) -> &[Point2D] {
        &self.control_points
    }

    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn knots(&self) -> &[f64] {
        &self.knots
    }

    pub fn evaluate(&self, u: f64) -> Point2D {
        // De Boor の rational 拡張は後続で実装
        todo!("NURBS評価は後続ステップで実装")
    }

    pub fn intersection_with_line(&self, line: &Line2D) -> Vec<Point2D> {
        let mut result = vec![];

        // 初期値を複数サンプル（例：0.1, 0.5, 0.9）
        for &u0 in &[0.1, 0.5, 0.9] {
            if let Some(u) = newton_solve(
                |u| self.evaluate(u).distance_to_point_on_line(line),
                |u| self.evaluate_derivative(u).dot(&self.normal_to_line(line, u)),
                u0,
                20,
                EPSILON,
            ) {
                let pt = self.evaluate(u);
                if line.distance_to_point(&pt) < EPSILON {
                    result.push(pt);
                }
            }
        }

        result.dedup_by(|a, b| a.distance_to(b) < EPSILON);
        result
    }

    fn normal_to_line(&self, line: &Line2D, u: f64) -> Direction2D {
        let pt = self.evaluate(u);
        let proj = line.project_point(&pt);
        pt.sub(&proj).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::geometry::geom2d::{line::Line2D, point::Point2D};

    #[test]
    fn test_nurbs_line_intersection() {
        let curve = NurbsCurve2D::from_quadratic([
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 2.0),
            Point2D::new(2.0, 0.0),
        ]);
        let line = Line2D::new(Point2D::new(0.0, 1.0), Point2D::new(2.0, 1.0));
        let pts = curve.intersection_with_line(&line);
        assert_eq!(pts.len(), 2);
        for pt in pts {
            assert!((pt.y - 1.0).abs() < 1e-10);
        }
    }
}