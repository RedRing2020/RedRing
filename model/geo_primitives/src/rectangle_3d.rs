//! Rect3D 実装（任意平面上の矩形）

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::{Direction3D, Point2D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, PointClassification, PrimitiveKind, PrimitiveMetadata,
    Rect3DConstructor, Rect3DContainment, Rect3DProperties, Scalar,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect3D<T: Scalar> {
    origin: Point3D<T>,
    u_axis: Direction3D<T>,
    v_axis: Direction3D<T>,
    width: T,
    height: T,
}

impl<T: Scalar> PrimitiveMetadata for Rect3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Rectangle
    }
}

impl<T: Scalar> Rect3D<T> {
    /// 4つの角（原点から U 軸方向・対角・V 軸方向の順）
    fn corners(&self) -> [Point3D<T>; 4] {
        let u = self.u_axis.as_vector() * self.width;
        let v = self.v_axis.as_vector() * self.height;
        [
            self.origin,
            self.origin + u,
            self.origin + u + v,
            self.origin + v,
        ]
    }
}

impl<T: Scalar> Rect3DConstructor<T> for Rect3D<T> {
    fn new(
        origin: Point3D<T>,
        u_axis: Direction3D<T>,
        v_axis: Direction3D<T>,
        width: T,
        height: T,
    ) -> Option<Self> {
        if width < T::ZERO || height < T::ZERO {
            return None;
        }

        // V 軸から U 軸方向の成分を除く
        let v_orthogonal = v_axis.as_vector() - u_axis.as_vector() * v_axis.dot(u_axis);
        let v_axis = Direction3D::from_vector(v_orthogonal)?;

        Some(Self {
            origin,
            u_axis,
            v_axis,
            width,
            height,
        })
    }
}

impl<T: Scalar> Rect3DProperties<T> for Rect3D<T> {
    fn origin(&self) -> Point3D<T> {
        self.origin
    }

    fn u_axis(&self) -> Direction3D<T> {
        self.u_axis
    }

    fn v_axis(&self) -> Direction3D<T> {
        self.v_axis
    }

    fn normal(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.u_axis.as_vector().cross(self.v_axis.as_vector()))
            .expect("u_axis and v_axis should define a valid normal")
    }

    fn width(&self) -> T {
        self.width
    }

    fn height(&self) -> T {
        self.height
    }

    fn center(&self) -> Point3D<T> {
        let two = T::ONE + T::ONE;
        self.origin
            + self.u_axis.as_vector() * (self.width / two)
            + self.v_axis.as_vector() * (self.height / two)
    }
}

impl<T: Scalar> Rect3DContainment<T> for Rect3D<T> {
    fn contains_point(&self, point: Point3D<T>) -> bool {
        self.classify_point(point) != PointClassification::Outside
    }

    /// 辺までの距離が既定の許容誤差以内なら、平面外でも `OnBoundary` とする。それ以外は、平面からの距離が
    /// 許容誤差以内で、平面へ投影した点が矩形の内側にあれば `Inside`、それ以外を `Outside` とする。
    fn classify_point(&self, point: Point3D<T>) -> PointClassification {
        let tolerance = default_distance_tolerance::<T>();
        let relative = Vector3D::from_points(self.origin, point);
        let in_plane = Point2D::new(
            relative.dot(self.u_axis.as_vector()),
            relative.dot(self.v_axis.as_vector()),
        );
        let plane_distance = relative.dot(self.normal().as_vector());

        let rectangle = geo_core::Aabb2D::new(
            Point2D::new(T::ZERO, T::ZERO),
            Point2D::new(self.width, self.height),
        );
        let in_plane_edge_distance = rectangle.distance_to_boundary(in_plane);
        let edge_distance = (in_plane_edge_distance * in_plane_edge_distance
            + plane_distance * plane_distance)
            .sqrt();

        if edge_distance <= tolerance {
            PointClassification::OnBoundary
        } else if plane_distance.abs() <= tolerance && rectangle.contains_point(in_plane) {
            PointClassification::Inside
        } else {
            PointClassification::Outside
        }
    }
}
