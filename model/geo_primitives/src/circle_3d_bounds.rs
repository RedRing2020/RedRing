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
        let center = self.center_internal();
        let radius = self.radius_internal();
        let u = self.ref_direction_internal().as_vector();
        let v = self.normal_internal().as_vector().cross(&u);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Point3D, Vector3D};
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let circle = Circle3D::new(center, normal, 5.0).unwrap();

        assert_eq!(circle.primitive_kind(), PrimitiveKind::Circle);
        assert_eq!(circle.area(), 25.0 * std::f64::consts::PI);

        let aabb = circle.aabb().expect("Circle should have an AABB");
        // XY平面の円なので、Z方向の範囲は0
        assert!((aabb.min().z() - center.z()).abs() < 1e-10);
        assert!((aabb.max().z() - center.z()).abs() < 1e-10);
        assert!((aabb.min().x() - (center.x() - 5.0)).abs() < 1e-10);
        assert!((aabb.max().y() - (center.y() + 5.0)).abs() < 1e-10);
    }

    #[test]
    fn test_circle_3d_aabb_respects_plane_orientation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let circle = Circle3D::new(center, normal, 2.0).unwrap();

        let aabb = circle.aabb().expect("Circle should have an AABB");
        assert!((aabb.min().x() - center.x()).abs() < 1e-10);
        assert!((aabb.max().x() - center.x()).abs() < 1e-10);
        assert!((aabb.min().y() - (center.y() - 2.0)).abs() < 1e-10);
        assert!((aabb.max().z() - (center.z() + 2.0)).abs() < 1e-10);
    }
}
