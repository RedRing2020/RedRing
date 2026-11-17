//! 面が載る曲面
//!
//! パラメータ (u, v) の取り方は、それぞれの曲面の型の決まりに従う。曲面の自然な法線は ∂u × ∂v の向き。

use crate::{
    Point3D, TopoConicalSurface3D, TopoCylindricalSurface3D, TopoNurbsSurface3D, TopoPlane3D,
    TopoSphericalSurface3D, TopoTorusSurface3D, Vector3D,
};
use geo_contracts::{
    ConicalSurface3DEvaluation, ConicalSurface3DProperties, CylindricalSurface3DEvaluation,
    CylindricalSurface3DProperties, NurbsSurface3DEvaluation, Plane3DProperties, Scalar,
    SphericalSurface3DEvaluation, SphericalSurface3DProperties, TorusSurface3DEvaluation,
    TorusSurface3DProperties,
};

/// 面が載る曲面
#[derive(Debug, Clone)]
pub enum SurfaceRef<T: Scalar> {
    /// 平面（u・v は平面の座標）
    Plane(TopoPlane3D<T>),
    /// 円柱（u は周の角度、v は軸の方向の距離）
    Cylinder(TopoCylindricalSurface3D<T>),
    /// 円錐（u は周の角度、v は軸の方向の距離）
    Cone(TopoConicalSurface3D<T>),
    /// 球（u は経度、v は緯度）
    Sphere(TopoSphericalSurface3D<T>),
    /// トーラス（u は軸のまわりの角度、v は管のまわりの角度）
    Torus(TopoTorusSurface3D<T>),
    /// NURBS 曲面（u・v はノットベクトルの値）
    Nurbs(TopoNurbsSurface3D<T>),
}

impl<T: Scalar> SurfaceRef<T> {
    /// 境界なしで全体を面にできる、閉じた曲面か
    pub fn is_closed(&self) -> bool {
        matches!(self, Self::Sphere(_) | Self::Torus(_))
    }

    /// パラメータ (u, v) の点（NURBS 曲面で定義域の外なら `None`）
    pub fn point_at_uv(&self, u: T, v: T) -> Option<Point3D<T>> {
        match self {
            Self::Plane(plane) => Some(
                plane.origin() + plane.u_axis().as_vector() * u + plane.v_axis().as_vector() * v,
            ),
            Self::Cylinder(surface) => Some(surface.point_at_uv(u, v)),
            Self::Cone(surface) => Some(surface.point_at_uv(u, v)),
            Self::Sphere(surface) => Some(surface.point_at_uv(u, v)),
            Self::Torus(surface) => Some(surface.point_at_uv(u, v)),
            Self::Nurbs(surface) => surface.point_at_uv(u, v),
        }
    }

    /// u・v の周期（周期のない方向は `None`）
    pub fn periods(&self) -> (Option<T>, Option<T>) {
        match self {
            Self::Plane(_) | Self::Nurbs(_) => (None, None),
            Self::Cylinder(_) | Self::Cone(_) | Self::Sphere(_) => (Some(T::TAU), None),
            Self::Torus(_) => (Some(T::TAU), Some(T::TAU)),
        }
    }

    /// u によらず 1 点になる v（球の 2 つの極・円錐の頂点。小さい順）
    pub fn poles(&self) -> Vec<T> {
        match self {
            Self::Sphere(_) => vec![-T::PI / T::from_f64(2.0), T::PI / T::from_f64(2.0)],
            Self::Cone(surface) => {
                vec![(surface.apex() - surface.center()).dot(surface.axis().as_vector())]
            }
            _ => Vec::new(),
        }
    }

    /// パラメータ (u, v) の法線（∂u × ∂v の向きの単位ベクトル。円錐の頂点・NURBS 曲面の定義域の外などで決まらなければ
    /// `None`）
    pub fn normal_at_uv(&self, u: T, v: T) -> Option<Vector3D<T>> {
        let direction = match self {
            // 平面の u・v の軸は、直交する単位ベクトル
            Self::Plane(plane) => {
                return Some(plane.u_axis().as_vector().cross(plane.v_axis().as_vector()));
            }
            Self::Cylinder(surface) => surface.normal_at(u, v),
            Self::Cone(surface) => surface.normal_at(u, v)?,
            Self::Sphere(surface) => surface.normal_at(u, v),
            Self::Torus(surface) => surface.normal_at(u, v),
            Self::Nurbs(surface) => surface.normal_at(u, v)?,
        };
        Some(direction.as_vector())
    }

    /// 点 `point` が、u によらず 1 点になる v の線（球の極・円錐の頂点）にあれば、その v
    pub fn singular_v(&self, point: Point3D<T>, tolerance: T) -> Option<T> {
        match self {
            Self::Sphere(surface) => {
                let pole = surface.axis().as_vector() * surface.radius();
                if point.distance_to(surface.center() + pole) <= tolerance {
                    Some(T::PI / T::from_f64(2.0))
                } else if point.distance_to(surface.center() - pole) <= tolerance {
                    Some(-T::PI / T::from_f64(2.0))
                } else {
                    None
                }
            }
            Self::Cone(surface) => {
                let apex = surface.apex();
                (point.distance_to(apex) <= tolerance)
                    .then(|| (apex - surface.center()).dot(surface.axis().as_vector()))
            }
            _ => None,
        }
    }

    /// 点 `point` のパラメータ (u, v)（解析的な曲面だけ。周期のある方向は `[0, 2π)`）
    ///
    /// 点は曲面の上にある前提で、曲面から離れた点は曲面の上の近い点の値になる。
    pub(crate) fn uv_of(&self, point: Point3D<T>) -> Option<(T, T)> {
        match self {
            Self::Plane(plane) => {
                let offset = point - plane.origin();
                Some((
                    offset.dot(plane.u_axis().as_vector()),
                    offset.dot(plane.v_axis().as_vector()),
                ))
            }
            Self::Cylinder(surface) => Some(
                Frame::new(
                    surface.center(),
                    surface.axis().as_vector(),
                    surface.ref_direction().as_vector(),
                )
                .angle_and_height(point),
            ),
            Self::Cone(surface) => {
                let (u, v) = Frame::new(
                    surface.center(),
                    surface.axis().as_vector(),
                    surface.ref_direction().as_vector(),
                )
                .angle_and_height(point);
                // 頂点の先（半径が負）では、点は角度 u の反対の側にある
                let radius = surface.radius() + v * surface.semi_angle().to_radians().tan();
                Some((
                    if radius < T::ZERO {
                        periodic(u - T::PI)
                    } else {
                        u
                    },
                    v,
                ))
            }
            Self::Sphere(surface) => {
                let frame = Frame::new(
                    surface.center(),
                    surface.axis().as_vector(),
                    surface.ref_direction().as_vector(),
                );
                let (x, y, z) = frame.local(point);
                Some((periodic(y.atan2(x)), z.atan2((x * x + y * y).sqrt())))
            }
            Self::Torus(surface) => {
                let frame = Frame::new(
                    surface.center(),
                    surface.axis().as_vector(),
                    surface.ref_direction().as_vector(),
                );
                let (x, y, z) = frame.local(point);
                let radial = (x * x + y * y).sqrt() - surface.major_radius();
                Some((periodic(y.atan2(x)), periodic(z.atan2(radial))))
            }
            Self::Nurbs(_) => None,
        }
    }
}

/// 角度を `[0, 2π)` に収める
fn periodic<T: Scalar>(angle: T) -> T {
    if angle < T::ZERO {
        angle + T::TAU
    } else {
        angle
    }
}

/// 軸のまわりの曲面の座標系（X は角度 0 の方向、Y は 軸 × X）
struct Frame<T: Scalar> {
    origin: Point3D<T>,
    x: Vector3D<T>,
    y: Vector3D<T>,
    z: Vector3D<T>,
}

impl<T: Scalar> Frame<T> {
    fn new(origin: Point3D<T>, z: Vector3D<T>, x: Vector3D<T>) -> Self {
        Self {
            origin,
            x,
            y: z.cross(x),
            z,
        }
    }

    fn local(&self, point: Point3D<T>) -> (T, T, T) {
        let offset = point - self.origin;
        (offset.dot(self.x), offset.dot(self.y), offset.dot(self.z))
    }

    /// 軸のまわりの角度と、軸の方向の距離
    fn angle_and_height(&self, point: Point3D<T>) -> (T, T) {
        let (x, y, z) = self.local(point);
        (periodic(y.atan2(x)), z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::{
        ConicalSurface3DConstructor, CylindricalSurface3DConstructor, Plane3DConstructor,
        SphericalSurface3DConstructor, TorusSurface3DConstructor,
    };
    use geo_primitives::Angle;

    fn surfaces() -> Vec<SurfaceRef<f64>> {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        vec![
            SurfaceRef::Plane(
                TopoPlane3D::from_three_points(
                    origin,
                    Point3D::new(2.0, 2.0, 3.0),
                    Point3D::new(1.0, 3.0, 4.0),
                )
                .unwrap(),
            ),
            SurfaceRef::Cylinder(TopoCylindricalSurface3D::new_standard(origin, 2.0).unwrap()),
            SurfaceRef::Cone(
                TopoConicalSurface3D::new_standard(origin, 2.0, Angle::from_degrees(30.0)).unwrap(),
            ),
            SurfaceRef::Sphere(TopoSphericalSurface3D::new_standard(origin, 2.0).unwrap()),
            SurfaceRef::Torus(TopoTorusSurface3D::new_standard(origin, 5.0, 2.0).unwrap()),
        ]
    }

    #[test]
    fn the_normal_points_along_the_cross_product_of_the_parameter_derivatives() {
        let step = 1.0e-6;
        for surface in surfaces() {
            for (u, v) in [(0.3, 0.4), (2.0, -0.7), (4.5, 1.1)] {
                let point = |u: f64, v: f64| surface.point_at_uv(u, v).unwrap();
                let du = (point(u + step, v) - point(u - step, v)) * (0.5 / step);
                let dv = (point(u, v + step) - point(u, v - step)) * (0.5 / step);
                let expected = du.cross(dv).normalize();
                let normal = surface.normal_at_uv(u, v).unwrap();
                assert!((normal.length() - 1.0).abs() < 1.0e-9);
                assert!(
                    normal.dot(expected) > 1.0 - 1.0e-6,
                    "{surface:?} at ({u}, {v})"
                );
            }
        }
    }

    #[test]
    fn the_poles_are_the_singular_lines() {
        for surface in surfaces() {
            for pole in surface.poles() {
                let a = surface.point_at_uv(0.0, pole).unwrap();
                let b = surface.point_at_uv(2.0, pole).unwrap();
                assert!(a.distance_to(b) < 1.0e-9);
                assert_eq!(surface.singular_v(a, 1.0e-9), Some(pole));
            }
        }
        assert_eq!(surfaces()[3].poles().len(), 2);
        assert_eq!(surfaces()[2].poles().len(), 1);
    }
}
