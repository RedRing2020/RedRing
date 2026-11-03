//! Circle3D の Bounds 実装

use crate::{Circle3D, Point3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

fn extent_from_plane_axes<T: Scalar>(radius: T, axis_u: T, axis_v: T) -> T {
    ((radius * axis_u) * (radius * axis_u) + (radius * axis_v) * (radius * axis_v)).sqrt()
}

impl<T: Scalar> Bounded<T> for Circle3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let center = self.center;
        let radius = self.radius;
        let u = self.ref_direction.as_vector();
        let v = self.axis.as_vector().cross(u);

        let extent_x = extent_from_plane_axes(radius, u.x(), v.x());
        let extent_y = extent_from_plane_axes(radius, u.y(), v.y());
        let extent_z = extent_from_plane_axes(radius, u.z(), v.z());

        let min_point = Point3D::new(
            center.x() - extent_x,
            center.y() - extent_y,
            center.z() - extent_z,
        );
        let max_point = Point3D::new(
            center.x() + extent_x,
            center.y() + extent_y,
            center.z() + extent_z,
        );

        Some(Aabb3D::new(min_point, max_point))
    }
}
