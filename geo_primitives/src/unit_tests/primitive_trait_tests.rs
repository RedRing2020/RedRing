//! GeometricPrimitiveトレイトのテスト
//! プリミティブインターフェースのテスト

use crate::geometry3d::{BBox3D, Point3D};
use crate::traits::common::classification::PrimitiveKind;
use crate::traits::common::geometry_utils::*;
use crate::traits::common::primitive_trait::GeometricPrimitive;
use geo_foundation::abstract_types::geometry::BBox as BBoxTrait;

// Primitive trait tests
struct MockPrimitive {
    kind: PrimitiveKind,
    bbox: BBox3D<f64>,
    measure: Option<f64>,
}

impl GeometricPrimitive for MockPrimitive {
    type BBox = BBox3D<f64>;

    fn primitive_kind(&self) -> PrimitiveKind {
        self.kind
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bbox.clone()
    }

    fn measure(&self) -> Option<f64> {
        self.measure
    }
}

#[test]
fn test_geometric_primitive_interface() {
    let primitive = MockPrimitive {
        kind: PrimitiveKind::Point,
        bbox: BBox3D::new_from_tuples((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)),
        measure: Some(5.0),
    };

    assert_eq!(primitive.primitive_kind(), PrimitiveKind::Point);
    assert_eq!(primitive.measure(), Some(5.0));

    let bbox = primitive.bounding_box();
    assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.max(), Point3D::new(1.0, 1.0, 1.0));
}

#[test]
fn test_f64_min_max() {
    assert_eq!(f64_min(1.0, 2.0), 1.0);
    assert_eq!(f64_max(1.0, 2.0), 2.0);
    assert_eq!(f64_min(2.0, 1.0), 1.0);
    assert_eq!(f64_max(2.0, 1.0), 2.0);
}

#[test]
fn test_point3d_bounding_box() {
    let points = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(2.0, 1.0, 3.0),
        Point3D::new(1.0, 3.0, 1.0),
    ];

    let bbox = point3d_bounding_box(&points).unwrap();
    assert_eq!(bbox, (0.0, 0.0, 0.0, 2.0, 3.0, 3.0));
}

#[test]
fn test_point3d_centroid() {
    let points = vec![
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(3.0, 0.0, 0.0),
        Point3D::new(0.0, 3.0, 0.0),
    ];

    let centroid = point3d_centroid(&points).unwrap();
    assert_eq!(centroid.x(), 1.0);
    assert_eq!(centroid.y(), 1.0);
    assert_eq!(centroid.z(), 0.0);
}
