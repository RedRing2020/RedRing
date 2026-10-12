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
        let center = self.center;
        let radius = self.radius;
        let u = self.start_direction().as_vector();
        let v = self.normal().as_vector().cross(u);

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
