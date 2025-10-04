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

/// 2D点 (f64 内部表現)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D { coords: [f64;2] }

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self { Self { coords: [x,y] } }
    pub fn from_f64(x:f64,y:f64)->Self { Self::new(x,y) }
    pub fn origin() -> Self { Self::new(0.0,0.0) }
    pub fn x(&self)->f64 { self.coords[0] }
    pub fn y(&self)->f64 { self.coords[1] }
    pub fn distance_to(&self, other:&Self)->Scalar {
        let dx = self.x()-other.x();
        let dy = self.y()-other.y();
        Scalar::new((dx*dx + dy*dy).sqrt())
    }
    pub fn to_vector(&self)->Vector2D { Vector2D::from_f64(self.x(), self.y()) }
    pub fn midpoint(&self, other:&Self)->Self { Self::new((self.x()+other.x())*0.5,(self.y()+other.y())*0.5) }
}

impl TolerantEq for Point2D {
    fn tolerant_eq(&self, other:&Self, context:&ToleranceContext)->bool {
        (self.x()-other.x()).abs() <= context.linear && (self.y()-other.y()).abs() <= context.linear
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

    pub fn direction(&self) -> Vector2D { Vector2D::from_f64(self.end.x()-self.start.x(), self.end.y()-self.start.y()) }

    pub fn midpoint(&self) -> Point2D {
        self.start.midpoint(&self.end)
    }

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar {
        let v = self.direction();
    let w = Vector2D::from_f64(point.x()-self.start.x(), point.y()-self.start.y());
        let c1 = v.dot(&w); // f64
        if c1 <= 0.0 { return self.start.distance_to(point); }
        let c2 = v.dot(&v); // f64
        if c1 >= c2 { return self.end.distance_to(point); }
        let t = c1 / c2; // 0..1
        let pb = Point2D::new(
            self.start.x() + t * (self.end.x()-self.start.x()),
            self.start.y() + t * (self.end.y()-self.start.y()),
        );
        point.distance_to(&pb)
    }
}

impl ParametricCurve2D for LineSegment2D {
    fn evaluate(&self, t: Scalar) -> Point2D {
        let one_minus_t = Scalar::new(1.0) - t.clone();
        Point2D::new(
            (one_minus_t.clone()*Scalar::new(self.start.x()) + t.clone()*Scalar::new(self.end.x())).value(),
            (one_minus_t*Scalar::new(self.start.y()) + t*Scalar::new(self.end.y())).value(),
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
        Point2D::new(
            self.center.x() + self.radius * c,
            self.center.y() + self.radius * s,
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

    let mut min_x = self.vertices[0].x();
    let mut max_x = self.vertices[0].x();
    let mut min_y = self.vertices[0].y();
    let mut max_y = self.vertices[0].y();

        for vertex in &self.vertices[1..] {
            if vertex.x() < min_x { min_x = vertex.x(); }
            if vertex.x() > max_x { max_x = vertex.x(); }
            if vertex.y() < min_y { min_y = vertex.y(); }
            if vertex.y() > max_y { max_y = vertex.y(); }
        }

        (Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 面積（符号付き）
    pub fn signed_area(&self) -> Scalar {
        if self.vertices.len() < 3 {
            return Scalar::new(0.0);
        }

    let mut area_acc = 0.0f64;
        let n = self.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            area_acc += self.vertices[i].x()*self.vertices[j].y() - self.vertices[j].x()*self.vertices[i].y();
        }
        Scalar::new(area_acc / 2.0)
    }

    /// 面積（絶対値）
    pub fn area(&self) -> Scalar {
        Scalar::new(self.signed_area().value().abs())
    }
}


