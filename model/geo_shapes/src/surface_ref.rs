//! 面が載る曲面
//!
//! パラメータ (u, v) の取り方は、それぞれの曲面の型の決まりに従う。曲面の自然な法線は ∂u × ∂v の向き。

use geo_contracts::{
    ConicalSurface3DEvaluation, ConicalSurface3DProperties, CylindricalSurface3DEvaluation,
    CylindricalSurface3DProperties, NurbsSurface3DEvaluation, Plane3DProperties, Scalar,
    SphericalSurface3DEvaluation, SphericalSurface3DProperties, TorusSurface3DEvaluation,
    TorusSurface3DProperties,
};
use geo_core::{Point3D, Vector3D};
use geo_nurbs::NurbsSurface3D;
use geo_primitives::{
    ConicalSurface3D, CylindricalSurface3D, Plane3D, SphericalSurface3D, TorusSurface3D,
};

/// 面が載る曲面
#[derive(Debug, Clone)]
pub enum SurfaceRef<T: Scalar> {
    /// 平面（u・v は平面の座標）
    Plane(Plane3D<T>),
    /// 円柱（u は周の角度、v は軸の方向の距離）
    Cylinder(CylindricalSurface3D<T>),
    /// 円錐（u は周の角度、v は軸の方向の距離）
    Cone(ConicalSurface3D<T>),
    /// 球（u は経度、v は緯度）
    Sphere(SphericalSurface3D<T>),
    /// トーラス（u は軸のまわりの角度、v は管のまわりの角度）。管が軸と交わるトーラス（りんご形・レモン形）は、
    /// 球と同じく v に周期がなく、R + r cos v = 0 の 2 つの v が極になる（v は `[-極, 極]`）
    Torus(TorusSurface3D<T>),
    /// NURBS 曲面（u・v はノットベクトルの値）
    Nurbs(NurbsSurface3D<T>),
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
            Self::Torus(surface) => match torus_pole(surface) {
                Some(_) => (Some(T::TAU), None),
                None => (Some(T::TAU), Some(T::TAU)),
            },
        }
    }

    /// u によらず 1 点になる v（球の 2 つの極・円錐の頂点・管が軸と交わるトーラスの軸の上の 2 点。小さい順）
    pub fn poles(&self) -> Vec<T> {
        match self {
            Self::Sphere(_) => vec![-T::PI / T::from_f64(2.0), T::PI / T::from_f64(2.0)],
            Self::Cone(surface) => {
                vec![(surface.apex() - surface.center()).dot(surface.axis().as_vector())]
            }
            Self::Torus(surface) => {
                torus_pole(surface).map_or(Vec::new(), |pole| vec![-pole, pole])
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
            Self::Torus(surface) => {
                // 管が軸と交わるトーラスの軸の上の点は、円錐の頂点と同じくとがり、法線が決まらない
                let radius = surface.major_radius() + surface.minor_radius() * v.cos();
                if torus_pole(surface).is_some()
                    && radius.abs() <= geo_contracts::default_distance_tolerance::<T>()
                {
                    return None;
                }
                surface.normal_at(u, v)
            }
            Self::Nurbs(surface) => surface.normal_at(u, v)?,
        };
        Some(direction.as_vector())
    }

    /// 点 `point` が、u によらず 1 点になる v の線（球の極・円錐の頂点・管が軸と交わるトーラスの軸の上の点）に
    /// あれば、その v
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
            Self::Torus(surface) => {
                let pole = torus_pole(surface)?;
                // 軸の上の点の高さは r sin(極)
                let tip = surface.axis().as_vector() * (surface.minor_radius() * pole.sin());
                if point.distance_to(surface.center() + tip) <= tolerance {
                    Some(pole)
                } else if point.distance_to(surface.center() - tip) <= tolerance {
                    Some(-pole)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// 点 `point` のパラメータ (u, v)（解析的な曲面だけ。周期のある方向は `[0, 2π)`）
    ///
    /// 点は曲面の上にある前提で、曲面から離れた点は曲面の上の近い点の値になる。
    pub fn uv_of(&self, point: Point3D<T>) -> Option<(T, T)> {
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
                let v = z.atan2(radial);
                // 管が軸と交わるトーラスの v は、周期で折り返さず `[-極, 極]` に収める
                // （極の外の角度では、断面の弧の上で近い点は極の点）
                let v = match torus_pole(surface) {
                    Some(pole) => v.max(-pole).min(pole),
                    None => periodic(v),
                };
                Some((periodic(y.atan2(x)), v))
            }
            Self::Nurbs(_) => None,
        }
    }
}

/// 管が軸と交わるトーラス（大半径が小半径より小さい）の極の v（R + r cos v = 0 の正の v）。ほかは `None`
fn torus_pole<T: Scalar>(surface: &TorusSurface3D<T>) -> Option<T> {
    (surface.major_radius() < surface.minor_radius())
        .then(|| (-surface.major_radius() / surface.minor_radius()).acos())
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
mod tests;
