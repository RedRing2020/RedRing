
use std::any::Any;

use crate::geometry::geometry2d::{
    point::Point,
    vector::Vector,
};

use crate::geometry_trait::point_ops::PointOps;
use crate::analysis::consts::EPSILON;

use crate::analysis::numeric::{find_span, basis_functions, basis_function_derivatives};
use crate::geometry_kind::CurveKind2D;
use crate::geometry_trait::Curve2D;

#[derive(Debug, Clone, PartialEq)]
pub struct NurbsCurve {
    degree: usize,
    control_points: Vec<Point>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    domain: (f64, f64),

    is_rational: bool, // 重み付きかどうか
    is_uniform: bool,  // ノットが一様かどうか
}

impl NurbsCurve {
    pub fn new(degree: usize, control_points: Vec<Point>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        assert_eq!(control_points.len(), weights.len(), "制御点と重みの数が一致しません");
        assert!(knots.len() >= control_points.len() + degree + 1, "ノットベクトルが不足しています");

        let domain = (
            knots[degree],
            knots[knots.len() - degree - 1],
        );

        let is_rational = weights.iter().any(|w| (w - 1.0).abs() > EPSILON);
        let uniform_step = knots[1] - knots[0];
        let is_uniform = knots.windows(2).all(|w| (w[1] - w[0] - uniform_step).abs() < EPSILON);

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

    pub fn degree(&self) -> usize { self.degree }

    pub fn control_points(self) -> Vec<Point> { self.control_points }

    pub fn weights(self) -> Vec<f64> { self.weights }

    pub fn knots(self) -> Vec<f64> { self.knots }

    pub fn domain(&self) -> (f64, f64) { self.domain }

    pub fn is_rational(&self) -> bool {
        self.is_rational
    }

    pub fn is_uniform(&self) -> bool {
        self.is_uniform
    }

    pub fn evaluate_derivative(&self, u: f64) -> Vector {
        let n = self.control_points.len() - 1;
        let p = self.degree;
        let span = find_span(n, p, u, &self.knots);
        let N = basis_functions(span, u, p, &self.knots);
        let dN = basis_function_derivatives(span, u, p, &self.knots);

        let mut numerator = Point::ORIGIN;
        let mut denominator = 0.0;
        let mut d_numerator = Point::ORIGIN;
        let mut d_denominator = 0.0;

        for i in 0..=p {
            let index = span - p + i;
            let w = self.weights[index];
            let cp = self.control_points[index];
            let Ni = N[i];
            let dNi = dN[i];

            numerator = numerator.add_scaled(&cp, Ni * w);
            denominator += Ni * w;

            d_numerator = d_numerator.add_scaled(&cp, dNi * w);
            d_denominator += dNi * w;
        }

        let dw = d_denominator;
        let dwP = d_numerator;
        let wP = numerator;

        let tangent = dwP.sub(&wP.mul(dw / denominator)).div(denominator);
        Vector::new(tangent.x(), tangent.y())
    }
/*
    pub fn intersection_with_line(&self, line: &Line) -> IntersectionResult<Point> {
        let mut pts = vec![];
        let mut params = vec![];

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
                    pts.push(pt);
                    params.push(u);
                }
            }
        }

        pts.dedup_by(|a, b| a.distance_to(b) < EPSILON);

        let kind = match pts.len() {
            0 => IntersectionKind::None,
            1 => IntersectionKind::Tangent,
            _ => IntersectionKind::Point,
        };

        IntersectionResult {
            kind,
            points: pts,
            parameters: params,
            tolerance_used: EPSILON,
        }
    }

    fn normal_to_line(&self, line: &Line, u: f64) -> Direction {
        let pt = self.evaluate(u);
        let proj = line.project_point(&pt);
        pt.sub(&proj).normalize()
    }
*/
}

impl Curve2D for NurbsCurve {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn kind(&self) -> CurveKind2D {
        CurveKind2D::NurbsCurve
    }

    fn evaluate(&self, u: f64) -> Point {
        // De Boor の rational 拡張は後続で実装
        todo!("NURBS評価は後続ステップで実装")
    }

    fn derivative(&self, _: f64) -> Vector {
        todo!("NURBSの導関数は後続ステップで実装")
    }

    fn length(&self) -> f64 {
        todo!("NURBSの長さ計算は後続ステップで実装")
    }
}
