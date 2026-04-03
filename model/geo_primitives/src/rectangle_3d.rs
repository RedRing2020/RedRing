//! Rect3D Core 実装（任意平面）

use crate::{Direction3D, Point3D, Vector3D};
use geo_contracts::{
    Rect3DConstructor, Rect3DContainment, Rect3DDerived, Rect3DEvaluation, Rect3DProperties, Scalar,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect3D<T: Scalar> {
    origin: Point3D<T>,
    u_axis: Direction3D<T>,
    v_axis: Direction3D<T>,
    width: T,
    height: T,
}

impl<T: Scalar> Rect3D<T> {
    pub fn new(
        origin: Point3D<T>,
        u_axis: Vector3D<T>,
        v_axis: Vector3D<T>,
        width: T,
        height: T,
    ) -> Option<Self> {
        if width < T::ZERO || height < T::ZERO {
            return None;
        }

        let u = Direction3D::from_vector(u_axis)?;
        let v_raw = Direction3D::from_vector(v_axis)?;

        let projection = u.as_vector() * v_raw.as_vector().dot(&u.as_vector());
        let v_orthogonal = v_raw.as_vector() - projection;
        let v = Direction3D::from_vector(v_orthogonal)?;

        Some(Self {
            origin,
            u_axis: u,
            v_axis: v,
            width,
            height,
        })
    }

    pub fn origin_point(&self) -> Point3D<T> {
        self.origin
    }

    pub fn u_axis_dir(&self) -> Direction3D<T> {
        self.u_axis
    }

    pub fn v_axis_dir(&self) -> Direction3D<T> {
        self.v_axis
    }

    pub fn normal_dir(&self) -> Direction3D<T> {
        Direction3D::from_vector(self.u_axis.as_vector().cross(&self.v_axis.as_vector()))
            .expect("u_axis and v_axis should define a valid normal")
    }

    pub fn width_value(&self) -> T {
        self.width
    }

    pub fn height_value(&self) -> T {
        self.height
    }

    pub fn center_point(&self) -> Point3D<T> {
        let two = T::ONE + T::ONE;
        let center_offset = self.u_axis.as_vector() * (self.width / two)
            + self.v_axis.as_vector() * (self.height / two);
        Point3D::new(
            self.origin.x() + center_offset.x(),
            self.origin.y() + center_offset.y(),
            self.origin.z() + center_offset.z(),
        )
    }

    pub fn area(&self) -> T {
        self.width * self.height
    }

    pub fn resize(&mut self, width: T, height: T) -> bool {
        if width < T::ZERO || height < T::ZERO {
            return false;
        }
        self.width = width;
        self.height = height;
        true
    }

    pub fn distance_to_plane(&self, point: Point3D<T>) -> T {
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        relative.dot(&self.normal_dir().as_vector())
    }

    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        let relative = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );

        let distance = relative.dot(&self.normal_dir().as_vector()).abs();
        if distance > tolerance {
            return false;
        }

        let u = relative.dot(&self.u_axis.as_vector());
        let v = relative.dot(&self.v_axis.as_vector());

        u >= -tolerance
            && u <= self.width + tolerance
            && v >= -tolerance
            && v <= self.height + tolerance
    }

    pub fn corners(&self) -> [Point3D<T>; 4] {
        let p0 = self.origin;
        let p1 = Point3D::new(
            self.origin.x() + self.u_axis.x() * self.width,
            self.origin.y() + self.u_axis.y() * self.width,
            self.origin.z() + self.u_axis.z() * self.width,
        );
        let p3 = Point3D::new(
            self.origin.x() + self.v_axis.x() * self.height,
            self.origin.y() + self.v_axis.y() * self.height,
            self.origin.z() + self.v_axis.z() * self.height,
        );
        let p2 = Point3D::new(
            p1.x() + self.v_axis.x() * self.height,
            p1.y() + self.v_axis.y() * self.height,
            p1.z() + self.v_axis.z() * self.height,
        );
        [p0, p1, p2, p3]
    }
}

impl<T: Scalar> Rect3DConstructor<T> for Rect3D<T> {
    fn new(
        origin: (T, T, T),
        u_axis: (T, T, T),
        v_axis: (T, T, T),
        width: T,
        height: T,
    ) -> Option<Self> {
        Self::new(
            Point3D::new(origin.0, origin.1, origin.2),
            Vector3D::new(u_axis.0, u_axis.1, u_axis.2),
            Vector3D::new(v_axis.0, v_axis.1, v_axis.2),
            width,
            height,
        )
    }

    fn resize(&mut self, width: T, height: T) -> bool {
        Self::resize(self, width, height)
    }
}

impl<T: Scalar> Rect3DProperties<T> for Rect3D<T> {
    fn origin(&self) -> (T, T, T) {
        (self.origin.x(), self.origin.y(), self.origin.z())
    }

    fn u_axis(&self) -> (T, T, T) {
        (self.u_axis.x(), self.u_axis.y(), self.u_axis.z())
    }

    fn v_axis(&self) -> (T, T, T) {
        (self.v_axis.x(), self.v_axis.y(), self.v_axis.z())
    }

    fn normal(&self) -> (T, T, T) {
        let n = self.normal_dir();
        (n.x(), n.y(), n.z())
    }

    fn width(&self) -> T {
        self.width
    }

    fn height(&self) -> T {
        self.height
    }

    fn center(&self) -> (T, T, T) {
        let c = self.center_point();
        (c.x(), c.y(), c.z())
    }
}

impl<T: Scalar> Rect3DContainment<T> for Rect3D<T> {
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool {
        Self::contains_point(self, &Point3D::new(point.0, point.1, point.2), tolerance)
    }
}

impl<T: Scalar> Rect3DDerived<T> for Rect3D<T> {
    fn area(&self) -> T {
        Self::area(self)
    }

    fn corners(&self) -> [(T, T, T); 4] {
        let c = Self::corners(self);
        [
            (c[0].x(), c[0].y(), c[0].z()),
            (c[1].x(), c[1].y(), c[1].z()),
            (c[2].x(), c[2].y(), c[2].z()),
            (c[3].x(), c[3].y(), c[3].z()),
        ]
    }

    fn is_valid(&self) -> bool {
        self.width >= T::ZERO && self.height >= T::ZERO
    }
}

impl<T: Scalar> Rect3DEvaluation<T> for Rect3D<T> {
    fn distance_to_plane(&self, point: (T, T, T)) -> T {
        Self::distance_to_plane(self, Point3D::new(point.0, point.1, point.2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POINT_CONTAINMENT_TOLERANCE: f64 = 1e-9;

    #[test]
    fn contains_and_resize() {
        let mut rect = Rect3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::unit_x(),
            Vector3D::unit_y(),
            10.0,
            5.0,
        )
        .unwrap();

        assert!(rect.contains_point(&Point3D::new(5.0, 2.5, 0.0), POINT_CONTAINMENT_TOLERANCE,));
        assert!(!rect.contains_point(&Point3D::new(11.0, 2.5, 0.0), POINT_CONTAINMENT_TOLERANCE,));
        assert!(!rect.contains_point(&Point3D::new(5.0, 2.5, 0.1), 1e-3));

        assert!(rect.resize(20.0, 10.0));
        assert_eq!(rect.width_value(), 20.0);
        assert_eq!(rect.height_value(), 10.0);
        assert!(!rect.resize(-1.0, 10.0));
    }
}
