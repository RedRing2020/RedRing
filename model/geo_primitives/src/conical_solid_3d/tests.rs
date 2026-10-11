//! ConicalSolid3D のテスト

use super::*;
use crate::{Point3D, Vector3D};
use geo_contracts::TolerantEq;

#[test]
fn test_tolerant_eq() {
    let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
    let solid1 = ConicalSolid3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        ref_direction,
        5.0,
        10.0,
    )
    .unwrap();
    let solid2 = ConicalSolid3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        ref_direction,
        5.0,
        10.0,
    )
    .unwrap();
    let solid3 = ConicalSolid3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Vector3D::new(0.0, 0.0, 1.0),
        ref_direction,
        3.0,
        10.0,
    )
    .unwrap();

    assert!(solid1.tolerant_eq(&solid2, 0.01));
    assert!(!solid1.tolerant_eq(&solid3, 0.01));
}
