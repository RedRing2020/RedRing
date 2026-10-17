//! SphericalSolid3D の relation 回帰テスト

use crate::{Point3D, SphericalSolid3D};
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata, SphericalSolid3DConstructor};

#[test]
fn test_spherical_solid_foundation() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let solid = SphericalSolid3D::new_standard(center, 2.0).unwrap();

    assert_eq!(solid.primitive_kind(), PrimitiveKind::SphericalSolid);

    let bbox = solid.aabb().expect("should have aabb");
    assert_eq!(bbox.min(), Point3D::new(-1.0, 0.0, 1.0));
    assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 5.0));
}
