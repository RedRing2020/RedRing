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

/// 3D点 (f64 内部表現)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D { coords: [f64;3] }

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self { Self { coords: [x,y,z] } }
    pub fn from_f64(x:f64,y:f64,z:f64)->Self { Self::new(x,y,z) }
    pub fn origin() -> Self { Self::new(0.0,0.0,0.0) }
    pub fn x(&self)->f64 { self.coords[0] }
    pub fn y(&self)->f64 { self.coords[1] }
    pub fn z(&self)->f64 { self.coords[2] }
    pub fn to_vector(&self)->Vector3D { Vector3D::from_f64(self.x(), self.y(), self.z()) }

    pub fn distance_to(&self, other:&Self)->Scalar {
        let dx = self.x()-other.x();
        let dy = self.y()-other.y();
        let dz = self.z()-other.z();
        Scalar::new((dx*dx + dy*dy + dz*dz).sqrt())
    }

    pub fn midpoint(&self, other:&Self)->Self { Self::new((self.x()+other.x())*0.5,(self.y()+other.y())*0.5,(self.z()+other.z())*0.5) }

    pub fn centroid(points:&[Self])->Option<Self> {
        if points.is_empty() { return None; }
        let mut sx=0.0; let mut sy=0.0; let mut sz=0.0;
        for p in points { sx+=p.x(); sy+=p.y(); sz+=p.z(); }
        let inv = 1.0 / points.len() as f64;
        Some(Self::new(sx*inv, sy*inv, sz*inv))
    }
}

impl TolerantEq for Point3D {
    fn tolerant_eq(&self, other:&Self, ctx:&ToleranceContext)->bool {
        (self.x()-other.x()).abs() <= ctx.linear && (self.y()-other.y()).abs() <= ctx.linear && (self.z()-other.z()).abs() <= ctx.linear
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

    pub fn direction(&self) -> Vector3D { Vector3D::from_f64(self.end.x()-self.start.x(), self.end.y()-self.start.y(), self.end.z()-self.start.z()) }

    pub fn midpoint(&self) -> Point3D {
        self.start.midpoint(&self.end)
    }

    /// 最接近点への距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let direction = self.direction();
    let to_point = Vector3D::from_f64(point.x()-self.start.x(), point.y()-self.start.y(), point.z()-self.start.z());
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
            (one_minus_t.clone()*Scalar::new(self.start.x()) + t.clone()*Scalar::new(self.end.x())).value(),
            (one_minus_t.clone()*Scalar::new(self.start.y()) + t.clone()*Scalar::new(self.end.y())).value(),
            (one_minus_t*Scalar::new(self.start.z()) + t*Scalar::new(self.end.z())).value(),
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
        Point3D::new(
            self.center.x() + self.radius * c * self.basis_u.x() + self.radius * s * self.basis_v.x(),
            self.center.y() + self.radius * c * self.basis_u.y() + self.radius * s * self.basis_v.y(),
            self.center.z() + self.radius * c * self.basis_u.z() + self.radius * s * self.basis_v.z(),
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
        let u_axis = Vector3D::from_f64(p2.x()-p1.x(), p2.y()-p1.y(), p2.z()-p1.z());
        let v_axis = Vector3D::from_f64(p3.x()-p1.x(), p3.y()-p1.y(), p3.z()-p1.z());
        Self::new(p1.clone(), u_axis, v_axis)
    }

    pub fn origin(&self) -> &Point3D { &self.origin }
    pub fn normal(&self) -> &Direction3D { &self.normal }
    pub fn u_axis(&self) -> &Vector3D { &self.u_axis }
    pub fn v_axis(&self) -> &Vector3D { &self.v_axis }

    /// 点から平面への距離
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
    let to_point = Vector3D::from_f64(point.x()-self.origin.x(), point.y()-self.origin.y(), point.z()-self.origin.z());
    Scalar::new(to_point.dot(self.normal.as_vector()).abs())
    }

    /// 点の平面上への投影
    pub fn project_point(&self, point: &Point3D) -> Point3D {
        let to_point = Vector3D::from_f64((point.x.clone()-self.origin.x.clone()).value(), (point.y.clone()-self.origin.y.clone()).value(), (point.z.clone()-self.origin.z.clone()).value());
        let distance = to_point.dot(self.normal.as_vector());
        Point3D::new(
            point.x() - distance * self.normal.x(),
            point.y() - distance * self.normal.y(),
            point.z() - distance * self.normal.z(),
        )
    }
}

impl ParametricSurface for Plane {
    fn evaluate(&self, u: Scalar, v: Scalar) -> Point3D {
        Point3D::new(
            (Scalar::new(self.origin.x()) + u.clone()*Scalar::new(self.u_axis.x()) + v.clone()*Scalar::new(self.v_axis.x())).value(),
            (Scalar::new(self.origin.y()) + u.clone()*Scalar::new(self.u_axis.y()) + v.clone()*Scalar::new(self.v_axis.y())).value(),
            (Scalar::new(self.origin.z()) + u*Scalar::new(self.u_axis.z()) + v*Scalar::new(self.v_axis.z())).value(),
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
            (Scalar::new(self.center.x()) + self.radius.clone() * sin_u.clone() * cos_v).value(),
            (Scalar::new(self.center.y()) + self.radius.clone() * sin_u * sin_v).value(),
            (Scalar::new(self.center.z()) + self.radius.clone() * cos_u).value(),
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
        let direction = Vector3D::from_f64(point.x()-self.center.x(), point.y()-self.center.y(), point.z()-self.center.z());
        let context = ToleranceContext::standard();
        Direction3D::from_vector(direction, &context)
    }
}


