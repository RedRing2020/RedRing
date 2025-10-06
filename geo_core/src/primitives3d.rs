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

// Point3D now lives in point3d.rs (always-on); keep ParametricCurve3D signatures unchanged.

/// 3D線分 (deprecated: move to geo_primitives::geometry3d::LineSegment3D)
#[deprecated(note = "Use geo_primitives::geometry3d::LineSegment3D; this core copy will be removed")]
#[derive(Debug, Clone)]
pub struct LineSegment3D {
    start: Point3D,
    end: Point3D,
}

#[allow(deprecated)]
impl LineSegment3D {
    pub fn new(start: Point3D, end: Point3D) -> Self { Self { start, end } }
    pub fn start(&self) -> &Point3D { &self.start }
    pub fn end(&self) -> &Point3D { &self.end }
    pub fn direction(&self) -> Vector3D {
        Vector3D::new(
            self.end.x.clone() - self.start.x.clone(),
            self.end.y.clone() - self.start.y.clone(),
            self.end.z.clone() - self.start.z.clone(),
        )
    }
    pub fn midpoint(&self) -> Point3D { self.start.midpoint(&self.end) }
    pub fn distance_to_point(&self, point: &Point3D) -> Scalar {
        let direction = self.direction();
        let to_point = Vector3D::new(
            point.x.clone() - self.start.x.clone(),
            point.y.clone() - self.start.y.clone(),
            point.z.clone() - self.start.z.clone(),
        );
        let length_sq = direction.dot(&direction);
        if length_sq.value().abs() < 1e-12 { return self.start.distance_to(point); }
        let t = to_point.dot(&direction) / length_sq;
        let t_clamped = if t.value() < 0.0 { Scalar::new(0.0) } else if t.value() > 1.0 { Scalar::new(1.0) } else { t };
        let closest = self.evaluate(t_clamped);
        closest.distance_to(point)
    }
}

#[allow(deprecated)]
impl ParametricCurve3D for LineSegment3D {
    fn evaluate(&self, t: Scalar) -> Point3D {
        let one_minus_t = Scalar::new(1.0) - t.clone();
        Point3D::new(
            one_minus_t.clone() * self.start.x.clone() + t.clone() * self.end.x.clone(),
            one_minus_t.clone() * self.start.y.clone() + t.clone() * self.end.y.clone(),
            one_minus_t * self.start.z.clone() + t * self.end.z.clone(),
        )
    }
    fn derivative(&self, _t: Scalar) -> Vector3D { self.direction() }
    fn parameter_bounds(&self) -> (Scalar, Scalar) { (Scalar::new(0.0), Scalar::new(1.0)) }
    fn length(&self) -> Scalar { self.start.distance_to(&self.end) }
}

/// 平面 (deprecated: move to geo_primitives::geometry3d::Plane)
#[deprecated(note = "Use geo_primitives::geometry3d::Plane; this core copy will be removed")]
#[derive(Debug, Clone)]
pub struct Plane {
    origin: Point3D,
    u_axis: Vector3D,
    v_axis: Vector3D,
    normal: Direction3D,
}

#[allow(deprecated)]
impl Plane {
    pub fn new(origin: Point3D, u_axis: Vector3D, v_axis: Vector3D) -> Option<Self> {
        let context = ToleranceContext::standard();
        let normal = Direction3D::from_vector(u_axis.cross(&v_axis), &context)?;
        Some(Self { origin, u_axis, v_axis, normal })
    }
    pub fn from_point_and_normal(origin: Point3D, normal: Direction3D) -> Self {
        let context = ToleranceContext::standard();
        let (u_axis, v_axis) = normal.orthonormal_basis(&context);
        Self { origin, u_axis, v_axis, normal }
    }
    pub fn from_three_points(p1:&Point3D,p2:&Point3D,p3:&Point3D)->Option<Self>{
        let u_axis = Vector3D::new(
            p2.x.clone() - p1.x.clone(),
            p2.y.clone() - p1.y.clone(),
            p2.z.clone() - p1.z.clone(),
        );
        let v_axis = Vector3D::new(
            p3.x.clone() - p1.x.clone(),
            p3.y.clone() - p1.y.clone(),
            p3.z.clone() - p1.z.clone(),
        );
        Self::new(p1.clone(), u_axis, v_axis)
    }
    pub fn origin(&self)->&Point3D { &self.origin }
    pub fn normal(&self)->&Direction3D { &self.normal }
    pub fn u_axis(&self)->&Vector3D { &self.u_axis }
    pub fn v_axis(&self)->&Vector3D { &self.v_axis }
    pub fn distance_to_point(&self, point:&Point3D)->Scalar {
        let to_point = Vector3D::new(
            point.x.clone() - self.origin.x.clone(),
            point.y.clone() - self.origin.y.clone(),
            point.z.clone() - self.origin.z.clone(),
        );
        to_point.dot(self.normal.as_vector()).abs()
    }
    pub fn project_point(&self, point:&Point3D)->Point3D {
        let to_point = Vector3D::new(
            point.x.clone() - self.origin.x.clone(),
            point.y.clone() - self.origin.y.clone(),
            point.z.clone() - self.origin.z.clone(),
        );
        let distance = to_point.dot(self.normal.as_vector());
        Point3D::new(
            point.x.clone() - distance.clone()*self.normal.x().clone(),
            point.y.clone() - distance.clone()*self.normal.y().clone(),
            point.z.clone() - distance*self.normal.z().clone(),
        )
    }
}

impl ParametricSurface for Plane {
    fn evaluate(&self, u: Scalar, v: Scalar) -> Point3D {
        Point3D::new(
            self.origin.x.clone() + u.clone() * self.u_axis.x().clone() + v.clone() * self.v_axis.x().clone(),
            self.origin.y.clone() + u.clone() * self.u_axis.y().clone() + v.clone() * self.v_axis.y().clone(),
            self.origin.z.clone() + u * self.u_axis.z().clone() + v * self.v_axis.z().clone(),
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

        Vector3D::new(
            self.radius.clone() * cos_u.clone() * cos_v,
            self.radius.clone() * cos_u * sin_v,
            -self.radius.clone() * sin_u,
        )
    }

    fn partial_v(&self, u: Scalar, v: Scalar) -> Vector3D {
        let sin_u = u.sin();
        let sin_v = v.sin();
        let cos_v = v.cos();

        Vector3D::new(
            -self.radius.clone() * sin_u.clone() * sin_v,
            self.radius.clone() * sin_u * cos_v,
            Scalar::new(0.0),
        )
    }

    fn parameter_bounds_u(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(std::f64::consts::PI))
    }

    fn parameter_bounds_v(&self) -> (Scalar, Scalar) {
        (Scalar::new(0.0), Scalar::new(2.0 * std::f64::consts::PI))
    }

    fn normal(&self, u: Scalar, v: Scalar) -> Option<Direction3D> {
        let point = self.evaluate(u, v);
        let direction = Vector3D::new(
            point.x.clone() - self.center.x.clone(),
            point.y.clone() - self.center.y.clone(),
            point.z.clone() - self.center.z.clone(),
        );
        let context = ToleranceContext::standard();
        Direction3D::from_vector(direction, &context)
    }
}


