//! Arc2D の Bounds 実装

use crate::{Arc2D, Point2D};
use geo_contracts::{Arc2DTrimRange, Bounded, Scalar};
use geo_core::Aabb2D;

impl<T: Scalar> Bounded<T> for Arc2D<T> {
    type Aabb = Aabb2D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let center = self.center_internal();
        let radius = self.radius_internal();

        if self.is_full_circle() {
            return Some(Aabb2D::new(
                Point2D::new(center.x() - radius, center.y() - radius),
                Point2D::new(center.x() + radius, center.y() + radius),
            ));
        }

        let mut points = vec![self.start_point(), self.end_point()];
        let half_pi = T::PI / (T::ONE + T::ONE);
        let critical_angles = [T::ZERO, half_pi, T::PI, T::PI + half_pi];

        for angle in critical_angles {
            if Arc2DTrimRange::contains_angle(self, angle) {
                points.push(Point2D::new(
                    center.x() + radius * angle.cos(),
                    center.y() + radius * angle.sin(),
                ));
            }
        }

        Aabb2D::from_points(&points)
    }
}
