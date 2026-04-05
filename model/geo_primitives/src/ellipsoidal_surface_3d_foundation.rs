//! EllipsoidalSurface3D の Foundation パターン実装

use crate::EllipsoidalSurface3D;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata, Scalar};
use geo_core::Aabb3D;

impl<T: Scalar> PrimitiveMetadata for EllipsoidalSurface3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::EllipsoidalSurface
    }
}

impl<T: Scalar> Bounded<T> for EllipsoidalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EllipsoidalSurface3D, Point3D, Vector3D};

    #[test]
    fn test_ellipsoidal_surface_foundation() {
        let surface = EllipsoidalSurface3D::new_standard((1.0, 2.0, 3.0), 2.0, 3.0, 4.0).unwrap();
        assert_eq!(surface.primitive_kind(), PrimitiveKind::EllipsoidalSurface);
        assert!(surface.surface_area() > 0.0);

        let bbox = surface.aabb().unwrap();
        assert!(bbox.max().x() > bbox.min().x());
        assert!(bbox.max().y() > bbox.min().y());
        assert!(bbox.max().z() > bbox.min().z());
    }

    #[test]
    fn test_ellipsoidal_surface_aabb_rotated() {
        let surface = EllipsoidalSurface3D::new(
            Point3D::origin(),
            Vector3D::new(0.0, 1.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
            1.0,
            3.0,
        )
        .unwrap();

        let bbox = surface.aabb().unwrap();
        assert!(!bbox.is_empty());
    }
}
