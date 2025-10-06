//! # 2次元幾何プリミティブ - 廃止予定
//!
//! **⚠️ このモジュールは廃止予定です。`geo_primitives` クレートをご利用ください。**
//!
//! geo_coreは数値計算・許容誤差・ロバスト判定に特化し、
//! 幾何プリミティブはgeo_primitivesに統合されました。

use crate::tolerance::{ToleranceContext, TolerantEq};
use crate::scalar::Scalar;
use crate::vector::{Vector, Vector2D};

/// パラメトリック曲線トレイト（2D）
#[deprecated(note = "Use geo_primitives instead")]
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
#[deprecated(note = "Use geo_primitives::Point2D instead")]
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

    pub fn to_vector(&self) -> Vector2D {
        Vector2D::new(self.x.clone(), self.y.clone())
    }

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
#[deprecated(note = "Use geo_primitives::LineSegment2D instead")]
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

    pub fn direction(&self) -> Vector2D {
        Vector2D::new(
            self.end.x.clone() - self.start.x.clone(),
            self.end.y.clone() - self.start.y.clone(),
        )
    }

    pub fn midpoint(&self) -> Point2D {
        self.start.midpoint(&self.end)
    }

    /// 点から線分への最短距離
    pub fn distance_to_point(&self, point: &Point2D) -> Scalar {
        let v = self.direction();
        let w = Vector2D::new(
            point.x.clone() - self.start.x.clone(),
            point.y.clone() - self.start.y.clone(),
        );

        let c1 = v.dot(&w);
        if c1.value() <= 0.0 {
            return self.start.distance_to(point);
        }

        let c2 = v.dot(&v);
        if c1.value() >= c2.value() {
            return self.end.distance_to(point);
        }

        let b = c1 / c2;
        let pb = Point2D::new(
            self.start.x.clone() + b.clone() * v.x(),
            self.start.y.clone() + b * v.y(),
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
#[deprecated(note = "Use geo_primitives::Arc2D instead")]
pub struct Arc2D {
    center: Point2D,
    radius: Scalar,
    start_angle: Scalar,
    end_angle: Scalar,
}

impl Arc2D {
    pub fn new(center: Point2D, radius: Scalar, start_angle: Scalar, end_angle: Scalar) -> Self {
        Self { center, radius, start_angle, end_angle }
    }

    pub fn center(&self) -> &Point2D { &self.center }
    pub fn radius(&self) -> &Scalar { &self.radius }
    pub fn start_angle(&self) -> &Scalar { &self.start_angle }
    pub fn end_angle(&self) -> &Scalar { &self.end_angle }

    /// 円弧の角度範囲
    pub fn angle_span(&self) -> Scalar {
        self.end_angle.clone() - self.start_angle.clone()
    }

    /// 中点での角度
    pub fn mid_angle(&self) -> Scalar {
        let two = Scalar::new(2.0);
        (self.start_angle.clone() + self.end_angle.clone()) / two
    }
}

impl ParametricCurve2D for Arc2D {
    fn evaluate(&self, t: Scalar) -> Point2D {
        let angle = self.start_angle.clone() + t * (self.end_angle.clone() - self.start_angle.clone());
        Point2D::new(
            self.center.x.clone() + self.radius.clone() * angle.cos(),
            self.center.y.clone() + self.radius.clone() * angle.sin(),
        )
    }

    fn derivative(&self, t: Scalar) -> Vector2D {
        let angle = self.start_angle.clone() + t * (self.end_angle.clone() - self.start_angle.clone());
        let angle_derivative = self.end_angle.clone() - self.start_angle.clone();
        Vector2D::new(
            -self.radius.clone() * angle.sin() * angle_derivative.clone(),
            self.radius.clone() * angle.cos() * angle_derivative,
        )
    }

    fn parameter_bounds(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(1.0))
    }

    fn length(&self) -> Scalar {
        self.radius.clone() * (self.end_angle.clone() - self.start_angle.clone()).abs()
    }
}

/// 2D多角形
#[derive(Debug, Clone)]
#[deprecated(note = "Use geo_primitives::Polygon2D instead")]
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


