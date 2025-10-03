/// 統合テスト（ライブラリ全体）

use crate::{Point3D, Scalar, DEFAULT_TOLERANCE};

#[test]
fn test_basic_integration() {
    let p1 = Point3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(0.0));
    let p2 = Point3D::new(Scalar::new(1.0), Scalar::new(0.0), Scalar::new(0.0));

    let distance = p1.distance_to(&p2);
    assert!((distance - Scalar::new(1.0)).abs().value() < DEFAULT_TOLERANCE.linear);
}