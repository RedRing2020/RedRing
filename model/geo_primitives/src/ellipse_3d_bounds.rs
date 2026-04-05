//! Ellipse3D の Bounds 実装

use crate::{Ellipse3D, Point3D};
use geo_contracts::{Bounded, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> Bounded<T> for Ellipse3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        let center = self.center_internal();
        let major = self.major_axis_direction().as_vector();
        let minor = self.minor_axis_direction().as_vector();
        let semi_major = self.semi_major_internal();
        let semi_minor = self.semi_minor_internal();

        let x_extent = ((semi_major * major.x()) * (semi_major * major.x())
            + (semi_minor * minor.x()) * (semi_minor * minor.x()))
        .sqrt();
        let y_extent = ((semi_major * major.y()) * (semi_major * major.y())
            + (semi_minor * minor.y()) * (semi_minor * minor.y()))
        .sqrt();
        let z_extent = ((semi_major * major.z()) * (semi_major * major.z())
            + (semi_minor * minor.z()) * (semi_minor * minor.z()))
        .sqrt();

        Some(Aabb3D::new(
            Point3D::new(
                center.x() - x_extent,
                center.y() - y_extent,
                center.z() - z_extent,
            ),
            Point3D::new(
                center.x() + x_extent,
                center.y() + y_extent,
                center.z() + z_extent,
            ),
        ))
    }
}
