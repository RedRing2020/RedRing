/// 2次元幾何プリミティブ
///
/// 2D空間での基本的な幾何要素（点、曲線）の定義と実装。

use crate::tolerance::{ToleranceContext, TolerantEq};
use crate::scalar::Scalar;
use crate::angle::Angle;
use crate::vector::{Vector, Vector2D};

/// パラメトリック曲線トレイト（2D）
pub trait ParametricCurve2D {
    fn evaluate(&self, t: Scalar) -> Point2D;
    fn derivative(&self, t: Scalar) -> Vector2D;
    fn parameter_bounds(&self) -> (Scalar, Scalar);
    fn length(&self) -> Scalar;
}

/// 2D点
/// 
/// 座標値はmm単位で格納される
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    x: Scalar,
    y: Scalar,
}

impl Point2D {
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub fn from_f64(x: f64, y: f64) -> Self {
        Self {
            x: Scalar::new(x),
            y: Scalar::new(y),
        }
    }

    pub fn origin() -> Self {
        Self::from_f64(0.0, 0.0)
    }

    pub fn x(&self) -> &Scalar { &self.x }
    pub fn y(&self) -> &Scalar { &self.y }

    pub fn distance_to(&self, other: &Self) -> Scalar {
        let dx = self.x.clone() - other.x.clone();
        let dy = self.y.clone() - other.y.clone();
        (dx.clone() * dx + dy.clone() * dy).sqrt()
    }

    pub fn to_vector(&self) -> Vector2D { Vector2D::from_f64(self.x.value(), self.y.value()) }

    pub fn midpoint(&self, other: &Self) -> Self {
        let two = Scalar::new(2.0);
        Self::new(
            (self.x.clone() + other.x.clone()) / two.clone(),
            (self.y.clone() + other.y.clone()) / two,
        )
    }
}

impl TolerantEq for Point2D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x.tolerant_eq(&other.x, context) && self.y.tolerant_eq(&other.y, context)
    }
}

/// 2D線分
#[derive(Debug, Clone)]
pub struct LineSegment2D {
    start: Point2D,
    end: Point2D,
}

impl LineSegment2D {
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> &Point2D { &self.start }
    pub fn end(&self) -> &Point2D { &self.end }

    pub fn direction(&self) -> Vector2D { Vector2D::from_f64((self.end.x.clone() - self.start.x.clone()).value(), (self.end.y.clone() - self.start.y.clone()).value()) }

    pub fn midpoint(&self) -> Point2D {
        self.start.midpoint(&self.end)
    }

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar {
        let v = self.direction();
        let w = Vector2D::from_f64((point.x.clone() - self.start.x.clone()).value(), (point.y.clone() - self.start.y.clone()).value());
        let c1 = v.dot(&w); // f64
        if c1 <= 0.0 { return self.start.distance_to(point); }
        let c2 = v.dot(&v); // f64
        if c1 >= c2 { return self.end.distance_to(point); }
        let b = Scalar::new(c1 / c2);
        let pb = Point2D::new(
            self.start.x.clone() + b.clone() * Scalar::new(v.x()),
            self.start.y.clone() + b * Scalar::new(v.y()),
        );
        point.distance_to(&pb)
    }
}

impl ParametricCurve2D for LineSegment2D {
    fn evaluate(&self, t: Scalar) -> Point2D {
        let one_minus_t = Scalar::new(1.0) - t.clone();
        Point2D::new(
            one_minus_t.clone() * self.start.x.clone() + t.clone() * self.end.x.clone(),
            one_minus_t * self.start.y.clone() + t * self.end.y.clone(),
        )
    }

    fn derivative(&self, _t: Scalar) -> Vector2D {
        self.direction()
    }

    fn parameter_bounds(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(1.0))
    }

    fn length(&self) -> Scalar {
        self.start.distance_to(&self.end)
    }
}

/// 円弧（2D）
#[derive(Debug, Clone)]
pub struct Arc2D {
    center: Point2D,
    radius: f64,
    start: Angle,
    end: Angle,
}

impl Arc2D {
    /// 新しい f64/Angle ベース円弧。角度は内部で (-π, π] 正規化される。
    pub fn new_f64(center: Point2D, radius: f64, start: Angle, end: Angle) -> Self {
        Self { center, radius, start: start.normalized(), end: end.normalized() }
    }

    /// 旧 Scalar ベース API (移行期間用)
    #[deprecated(note = "Use new_f64 with Angle instead of Scalar angles")] 
    pub fn new(center: Point2D, radius: Scalar, start_angle: Scalar, end_angle: Scalar) -> Self {
        Self::new_f64(center, radius.value(), Angle::from_radians(start_angle.value()), Angle::from_radians(end_angle.value()))
    }

    pub fn center(&self) -> &Point2D { &self.center }
    pub fn radius_f64(&self) -> f64 { self.radius }
    pub fn start_angle(&self) -> Angle { self.start }
    pub fn end_angle(&self) -> Angle { self.end }

    #[deprecated(note = "Use radius_f64() instead")] 
    pub fn radius(&self) -> Scalar { Scalar::new(self.radius) }
    #[deprecated(note = "Use start_angle() returning Angle")] 
    pub fn start_angle_scalar(&self) -> Scalar { Scalar::new(self.start.radians()) }
    #[deprecated(note = "Use end_angle() returning Angle")] 
    pub fn end_angle_scalar(&self) -> Scalar { Scalar::new(self.end.radians()) }

    pub fn angle_span_f64(&self) -> f64 { (self.end.delta(self.start)).radians().abs() }
    pub fn mid_angle(&self) -> Angle { Angle::lerp(self.start, self.end, 0.5) }
}

impl ParametricCurve2D for Arc2D {
    fn evaluate(&self, t: Scalar) -> Point2D {
        // t in [0,1] map to start..end along shortest path
        let span = self.end.delta(self.start); // (-π, π]
    let theta = self.start.radians() + span.radians() * t.value();
        let (s, c) = theta.sin_cos();
        Point2D::from_f64(
            self.center.x().value() + self.radius * c,
            self.center.y().value() + self.radius * s,
        )
    }
    fn derivative(&self, t: Scalar) -> Vector2D {
        let span = self.end.delta(self.start);
    let theta = self.start.radians() + span.radians() * t.value();
        let (s, c) = theta.sin_cos();
        let dtheta_dt = span.radians();
        // d/dt (r(c,s)) = r * dθ/dt * (-s, c)
        Vector2D::from_f64(-self.radius * s * dtheta_dt, self.radius * c * dtheta_dt)
    }
    fn parameter_bounds(&self) -> (Scalar, Scalar) { (Scalar::new(0.0), Scalar::new(1.0)) }
    fn length(&self) -> Scalar { Scalar::new(self.angle_span_f64() * self.radius) }
}

/// 2D多角形
#[derive(Debug, Clone)]
pub struct Polygon2D {
    vertices: Vec<Point2D>,
}

impl Polygon2D {
    pub fn new(vertices: Vec<Point2D>) -> Self {
        Self { vertices }
    }

    pub fn vertices(&self) -> &[Point2D] {
        &self.vertices
    }

    pub fn is_closed(&self) -> bool {
        self.vertices.len() >= 3
    }

    /// 境界ボックス
    pub fn bounding_box(&self) -> (Point2D, Point2D) {
        if self.vertices.is_empty() {
            return (Point2D::origin(), Point2D::origin());
        }

        let mut min_x = self.vertices[0].x.clone();
        let mut max_x = self.vertices[0].x.clone();
        let mut min_y = self.vertices[0].y.clone();
        let mut max_y = self.vertices[0].y.clone();

        for vertex in &self.vertices[1..] {
            if vertex.x.value() < min_x.value() { min_x = vertex.x.clone(); }
            if vertex.x.value() > max_x.value() { max_x = vertex.x.clone(); }
            if vertex.y.value() < min_y.value() { min_y = vertex.y.clone(); }
            if vertex.y.value() > max_y.value() { max_y = vertex.y.clone(); }
        }

        (Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 面積（符号付き）
    pub fn signed_area(&self) -> Scalar {
        if self.vertices.len() < 3 {
            return Scalar::new(0.0);
        }

        let mut area = Scalar::new(0.0);
        let n = self.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area = area + (self.vertices[i].x.clone() * self.vertices[j].y.clone() -
                          self.vertices[j].x.clone() * self.vertices[i].y.clone());
        }

        area / Scalar::new(2.0)
    }

    /// 面積（絶対値）
    pub fn area(&self) -> Scalar {
        self.signed_area().abs()
    }
}


