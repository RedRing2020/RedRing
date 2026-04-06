//! Arc3D の Bounds 実装

use crate::{Arc3D, Point3D};
use geo_contracts::{Arc3DEndpoint, Arc3DTrimRange, Bounded, Scalar};
use geo_core::Aabb3D;

fn candidate_extreme_angles<T: Scalar>(axis_u: T, axis_v: T) -> [T; 2] {
    let angle = axis_v.atan2(axis_u);
    [angle, angle + T::PI]
}

impl<T: Scalar> Bounded<T> for Arc3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let center = self.center_internal();
        let radius = self.radius_internal();
        let u = self.start_direction().as_vector();
        let v = self.normal().as_vector().cross(&u);

        let start = Arc3DEndpoint::start_point(self);
        let end = Arc3DEndpoint::end_point(self);
        let mut points = vec![
            Point3D::new(start.0, start.1, start.2),
            Point3D::new(end.0, end.1, end.2),
        ];

        for angle in candidate_extreme_angles(u.x(), v.x())
            .into_iter()
            .chain(candidate_extreme_angles(u.y(), v.y()))
            .chain(candidate_extreme_angles(u.z(), v.z()))
        {
            if Arc3DTrimRange::contains_angle(self, angle) {
                points.push(Point3D::new(
                    center.x() + radius * (u.x() * angle.cos() + v.x() * angle.sin()),
                    center.y() + radius * (u.y() * angle.cos() + v.y() * angle.sin()),
                    center.z() + radius * (u.z() * angle.cos() + v.z() * angle.sin()),
                ));
            }
        }

        Aabb3D::from_points(&points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Point3D, Vector3D};
    use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

    #[test]
    fn test_metadata_and_bounds_capabilities() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let start_angle = geo_contracts::Angle::from_radians(0.0);
        let end_angle = geo_contracts::Angle::from_radians(std::f64::consts::PI);
        let arc = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();

        assert_eq!(arc.primitive_kind(), PrimitiveKind::Arc);
        assert_eq!(arc.length(), 5.0 * std::f64::consts::PI);

        let aabb = arc.aabb().expect("Arc should have an AABB");
        assert!((aabb.min().x() + 5.0).abs() < 1e-10);
        assert!((aabb.max().x() - 5.0).abs() < 1e-10);
        assert!((aabb.min().y() - 0.0).abs() < 1e-10);
        assert!((aabb.max().y() - 5.0).abs() < 1e-10);
        assert!((aabb.min().z() - 0.0).abs() < 1e-10);
        assert!((aabb.max().z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_arc_3d_aabb_respects_trimmed_range() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let arc = Arc3D::new(
            center,
            2.0,
            normal,
            start_dir,
            geo_contracts::Angle::from_radians(0.0),
            geo_contracts::Angle::from_radians(std::f64::consts::PI),
        )
        .unwrap();

        let aabb = arc.aabb().expect("Arc should have an AABB");
        assert!((aabb.min().x() + 2.0).abs() < 1e-10);
        assert!((aabb.max().x() - 2.0).abs() < 1e-10);
        assert!((aabb.min().y() - 0.0).abs() < 1e-10);
        assert!((aabb.max().y() - 0.0).abs() < 1e-10);
        assert!((aabb.min().z() + 2.0).abs() < 1e-10);
        assert!((aabb.max().z() - 0.0).abs() < 1e-10);
    }
}
