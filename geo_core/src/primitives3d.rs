/// 3次元幾何プリミティブ
///
/// 3D空間での基本的な幾何要素（点、曲線、サーフェス）の定義と実装。

use crate::tolerance::{ToleranceContext, TolerantEq};
use crate::scalar::Scalar;
use crate::vector::{Vector, Vector3D, Direction3D};

/// パラメトリック曲線トレイト（3D）
pub trait ParametricCurve3D {
    fn evaluate(&self, t: Scalar) -> Point3D;
    fn derivative(&self, t: Scalar) -> Vector3D;
    fn parameter_bounds(&self) -> (Scalar, Scalar);
    fn length(&self) -> Scalar;
}

/// パラメトリックサーフェストレイト
pub trait ParametricSurface {
    fn evaluate(&self, u: Scalar, v: Scalar) -> Point3D;
    fn partial_u(&self, u: Scalar, v: Scalar) -> Vector3D;
    fn partial_v(&self, u: Scalar, v: Scalar) -> Vector3D;
    fn parameter_bounds_u(&self) -> (Scalar, Scalar);
    fn parameter_bounds_v(&self) -> (Scalar, Scalar);
    fn normal(&self, u: Scalar, v: Scalar) -> Option<Direction3D>;
}

/// 3D点
#[derive(Debug, Clone)]
pub struct Point3D {
    x: Scalar,
    y: Scalar,
    z: Scalar,
}

impl Point3D {
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self { x, y, z }
    }

    pub fn from_f64(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Scalar::new(x),
            y: Scalar::new(y),
            z: Scalar::new(z),
        }
    }

    pub fn origin() -> Self {
        Self::from_f64(0.0, 0.0, 0.0)
    }

    pub fn x(&self) -> &Scalar { &self.x }
    pub fn y(&self) -> &Scalar { &self.y }
    pub fn z(&self) -> &Scalar { &self.z }

    pub fn distance_to(&self, other: &Self) -> Scalar {
        let dx = self.x.clone() - other.x.clone();
        let dy = self.y.clone() - other.y.clone();
        let dz = self.z.clone() - other.z.clone();
        (dx.clone() * dx + dy.clone() * dy + dz.clone() * dz).sqrt()
    }

    pub fn to_vector(&self) -> Vector3D { Vector3D::from_f64(self.x.value(), self.y.value(), self.z.value()) }

    pub fn midpoint(&self, other: &Self) -> Self {
        let two = Scalar::new(2.0);
        Self::new(
            (self.x.clone() + other.x.clone()) / two.clone(),
            (self.y.clone() + other.y.clone()) / two.clone(),
            (self.z.clone() + other.z.clone()) / two,
        )
    }

    /// 重心計算
    pub fn centroid(points: &[Self]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut sum_x = Scalar::new(0.0);
        let mut sum_y = Scalar::new(0.0);
        let mut sum_z = Scalar::new(0.0);

        for point in points {
            sum_x = sum_x + point.x.clone();
            sum_y = sum_y + point.y.clone();
            sum_z = sum_z + point.z.clone();
        }

        let count = Scalar::new(points.len() as f64);
        Some(Self::new(
            sum_x / count.clone(),
            sum_y / count.clone(),
            sum_z / count,
        ))
    }
}

impl TolerantEq for Point3D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.x.tolerant_eq(&other.x, context)
            && self.y.tolerant_eq(&other.y, context)
            && self.z.tolerant_eq(&other.z, context)
    }
}

/// 3D線分
#[derive(Debug, Clone)]
pub struct LineSegment3D {
    start: Point3D,
    end: Point3D,
}

impl LineSegment3D {
    pub fn new(start: Point3D, end: Point3D) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> &Point3D { &self.start }
    pub fn end(&self) -> &Point3D { &self.end }

    pub fn direction(&self) -> Vector3D { Vector3D::from_f64((self.end.x.clone()-self.start.x.clone()).value(), (self.end.y.clone()-self.start.y.clone()).value(), (self.end.z.clone()-self.start.z.clone()).value()) }

    pub fn midpoint(&self) -> Point3D {
        self.start.midpoint(&self.end)
    }

    /// 最接近点への距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let direction = self.direction();
        let to_point = Vector3D::from_f64((point.x.clone()-self.start.x.clone()).value(), (point.y.clone()-self.start.y.clone()).value(), (point.z.clone()-self.start.z.clone()).value());
        let length_sq = direction.dot(&direction); // f64
        if length_sq.abs() < 1e-12 { return self.start.distance_to(point); }
        let t = to_point.dot(&direction) / length_sq; // f64
        let t_clamped = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
        let tc = Scalar::new(t_clamped);
        let closest = self.evaluate(tc);
        closest.distance_to(point)
    }
}

impl ParametricCurve3D for LineSegment3D {
    fn evaluate(&self, t: Scalar) -> Point3D {
        let one_minus_t = Scalar::new(1.0) - t.clone();
        Point3D::new(
            one_minus_t.clone() * self.start.x.clone() + t.clone() * self.end.x.clone(),
            one_minus_t.clone() * self.start.y.clone() + t.clone() * self.end.y.clone(),
            one_minus_t * self.start.z.clone() + t * self.end.z.clone(),
        )
    }

    fn derivative(&self, _t: Scalar) -> Vector3D {
        self.direction()
    }

    fn parameter_bounds(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(1.0))
    }

    fn length(&self) -> Scalar {
        self.start.distance_to(&self.end)
    }
}

/// 3D円（平面内の閉曲線）
#[derive(Debug, Clone)]
pub struct Circle3D {
    center: Point3D,
    radius: f64,
    normal: Direction3D,
    basis_u: Vector3D,
    basis_v: Vector3D,
}

impl Circle3D {
    pub fn new(center: Point3D, radius: f64, normal: Direction3D) -> Self {
        debug_assert!(radius >= 0.0, "radius must be non-negative");
        // 基底生成: Z 軸付近は (1,0,0),(0,1,0) を明示採用して t=0 で +X を得る
        let ctx = ToleranceContext::standard();
        let (u, v) = if normal.z().abs() > 0.999_999 {
            (Vector3D::from_f64(1.0,0.0,0.0), Vector3D::from_f64(0.0,1.0,0.0))
        } else {
            normal.orthonormal_basis(&ctx)
        };
        Self { center, radius, normal, basis_u: u, basis_v: v }
    }

    pub fn center(&self) -> &Point3D { &self.center }
    pub fn radius(&self) -> f64 { self.radius }
    pub fn normal(&self) -> &Direction3D { &self.normal }
    pub fn basis(&self) -> (&Vector3D, &Vector3D) { (&self.basis_u, &self.basis_v) }

    /// f64 パラメトリック評価 (t in [0,1])
    pub fn evaluate_f64(&self, t: f64) -> Point3D {
        let theta = t * std::f64::consts::TAU; let (s,c)=theta.sin_cos();
        Point3D::from_f64(
            self.center.x().value() + self.radius * c * self.basis_u.x() + self.radius * s * self.basis_v.x(),
            self.center.y().value() + self.radius * c * self.basis_u.y() + self.radius * s * self.basis_v.y(),
            self.center.z().value() + self.radius * c * self.basis_u.z() + self.radius * s * self.basis_v.z(),
        )
    }

    pub fn derivative_f64(&self, t: f64) -> Vector3D {
        let theta = t * std::f64::consts::TAU; let (s,c)=theta.sin_cos(); let dtheta=std::f64::consts::TAU;
        // r dθ (-s u + c v)
        let term_u = self.basis_u.clone() * (-self.radius * s * dtheta);
        let term_v = self.basis_v.clone() * ( self.radius * c * dtheta);
        term_u + term_v
    }
}

impl ParametricCurve3D for Circle3D {
    fn evaluate(&self, t: Scalar) -> Point3D { self.evaluate_f64(t.value()) }
    fn derivative(&self, t: Scalar) -> Vector3D { self.derivative_f64(t.value()) }
    fn parameter_bounds(&self) -> (Scalar, Scalar) { (Scalar::new(0.0), Scalar::new(1.0)) }
    fn length(&self) -> Scalar { Scalar::new(self.radius * std::f64::consts::TAU) }
}

/// 平面
#[derive(Debug, Clone)]
pub struct Plane {
    origin: Point3D,
    u_axis: Vector3D,
    v_axis: Vector3D,
    normal: Direction3D,
}

impl Plane {
    pub fn new(origin: Point3D, u_axis: Vector3D, v_axis: Vector3D) -> Option<Self> {
        let context = ToleranceContext::standard();
        let normal = Direction3D::from_vector(u_axis.cross(&v_axis), &context)?;
        Some(Self { origin, u_axis, v_axis, normal })
    }

    pub fn from_point_and_normal(origin: Point3D, normal: Direction3D) -> Self {
        // 適当な直交基底を生成
        let context = ToleranceContext::standard();
        let (u_axis, v_axis) = normal.orthonormal_basis(&context);
        Self { origin, u_axis, v_axis, normal }
    }

    pub fn from_three_points(p1: &Point3D, p2: &Point3D, p3: &Point3D) -> Option<Self> {
        let u_axis = Vector3D::from_f64((p2.x.clone()-p1.x.clone()).value(), (p2.y.clone()-p1.y.clone()).value(), (p2.z.clone()-p1.z.clone()).value());
        let v_axis = Vector3D::from_f64((p3.x.clone()-p1.x.clone()).value(), (p3.y.clone()-p1.y.clone()).value(), (p3.z.clone()-p1.z.clone()).value());
        Self::new(p1.clone(), u_axis, v_axis)
    }

    pub fn origin(&self) -> &Point3D { &self.origin }
    pub fn normal(&self) -> &Direction3D { &self.normal }
    pub fn u_axis(&self) -> &Vector3D { &self.u_axis }
    pub fn v_axis(&self) -> &Vector3D { &self.v_axis }

    /// 点から平面への距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let to_point = Vector3D::from_f64((point.x.clone()-self.origin.x.clone()).value(), (point.y.clone()-self.origin.y.clone()).value(), (point.z.clone()-self.origin.z.clone()).value());
        Scalar::new(to_point.dot(self.normal.as_vector()).abs())
    }

    /// 点の平面上への投影
    pub fn project_point(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::from_f64((point.x.clone()-self.origin.x.clone()).value(), (point.y.clone()-self.origin.y.clone()).value(), (point.z.clone()-self.origin.z.clone()).value());
        let distance = to_point.dot(self.normal.as_vector());
        Point3D::new(
            point.x.clone() - Scalar::new(distance * self.normal.x()),
            point.y.clone() - Scalar::new(distance * self.normal.y()),
            point.z.clone() - Scalar::new(distance * self.normal.z()),
        )
    }
}

impl ParametricSurface for Plane {
    fn evaluate(&self, u: Scalar, v: Scalar) -> Point3D {
        Point3D::new(
            self.origin.x.clone() + u.clone() * Scalar::new(self.u_axis.x()) + v.clone() * Scalar::new(self.v_axis.x()),
            self.origin.y.clone() + u.clone() * Scalar::new(self.u_axis.y()) + v.clone() * Scalar::new(self.v_axis.y()),
            self.origin.z.clone() + u * Scalar::new(self.u_axis.z()) + v * Scalar::new(self.v_axis.z()),
        )
    }

    fn partial_u(&self, _u: Scalar, _v: Scalar) -> Vector3D {
        self.u_axis.clone()
    }

    fn partial_v(&self, _u: Scalar, _v: Scalar) -> Vector3D {
        self.v_axis.clone()
    }

    fn parameter_bounds_u(&self) -> (Scalar, Scalar) {
        (Scalar::new(f64::NEG_INFINITY), Scalar::new(f64::INFINITY))
    }

    fn parameter_bounds_v(&self) -> (Scalar, Scalar) {
        (Scalar::new(f64::NEG_INFINITY), Scalar::new(f64::INFINITY))
    }

    fn normal(&self, _u: Scalar, _v: Scalar) -> Option<Direction3D> {
        Some(self.normal.clone())
    }
}

/// 球面
#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3D,
    radius: Scalar,
}

impl Sphere {
    pub fn new(center: Point3D, radius: Scalar) -> Self {
        Self { center, radius }
    }

    pub fn center(&self) -> &Point3D { &self.center }
    pub fn radius(&self) -> &Scalar { &self.radius }

    /// 表面積
    pub fn surface_area(&self) -> Scalar {
        let four_pi = Scalar::new(4.0 * std::f64::consts::PI);
        four_pi * self.radius.clone() * self.radius.clone()
    }

    /// 体積
    pub fn volume(&self) -> Scalar {
        let four_thirds_pi = Scalar::new(4.0 * std::f64::consts::PI / 3.0);
        four_thirds_pi * self.radius.clone() * self.radius.clone() * self.radius.clone()
    }
}

impl ParametricSurface for Sphere {
    fn evaluate(&self, u: Scalar, v: Scalar) -> Point3D {
        // u: 緯度 (0 to π), v: 経度 (0 to 2π)
        let sin_u = u.sin();
        let cos_u = u.cos();
        let sin_v = v.sin();
        let cos_v = v.cos();

        Point3D::new(
            self.center.x.clone() + self.radius.clone() * sin_u.clone() * cos_v,
            self.center.y.clone() + self.radius.clone() * sin_u * sin_v,
            self.center.z.clone() + self.radius.clone() * cos_u,
        )
    }

    fn partial_u(&self, u: Scalar, v: Scalar) -> Vector3D {
        let cos_u = u.cos();
        let sin_u = u.sin();
        let sin_v = v.sin();
        let cos_v = v.cos();

        Vector3D::from_f64((self.radius.clone()*cos_u.clone()*cos_v).value(), (self.radius.clone()*cos_u*sin_v).value(), (-self.radius.clone()*sin_u).value())
    }

    fn partial_v(&self, u: Scalar, v: Scalar) -> Vector3D {
        let sin_u = u.sin();
        let sin_v = v.sin();
        let cos_v = v.cos();

        Vector3D::from_f64((-self.radius.clone()*sin_u.clone()*sin_v).value(), (self.radius.clone()*sin_u*cos_v).value(), 0.0)
    }

    fn parameter_bounds_u(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(std::f64::consts::PI))
    }

    fn parameter_bounds_v(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(2.0 * std::f64::consts::PI))
    }

    fn normal(&self, u: Scalar, v: Scalar) -> Option<Direction3D> {
        let point = self.evaluate(u, v);
        let direction = Vector3D::from_f64((point.x.clone()-self.center.x.clone()).value(), (point.y.clone()-self.center.y.clone()).value(), (point.z.clone()-self.center.z.clone()).value());
        let context = ToleranceContext::standard();
        Direction3D::from_vector(direction, &context)
    }
}


