#[cfg(test)]
mod traits_tests {
    use crate::traits::common::classification::{PrimitiveKind, DimensionClass};
    use crate::traits::common::primitive_trait::GeometricPrimitive;
    use crate::traits::common::geometry_utils::*;
    use crate::traits::bbox_trait::{BoundingBox, BoundingBoxOps};
    use crate::geometry2d::Point2D;
    use crate::geometry3d::{Point3D, BBox3D};

    // Classification tests
    #[test]
    fn test_dimension_classification() {
        assert_eq!(PrimitiveKind::Point.dimension(), DimensionClass::Zero);
        assert_eq!(PrimitiveKind::LineSegment.dimension(), DimensionClass::One);
        assert_eq!(PrimitiveKind::Circle.dimension(), DimensionClass::Two);
        assert_eq!(PrimitiveKind::Sphere.dimension(), DimensionClass::Three);
    }

    #[test]
    fn test_property_checks() {
        assert!(PrimitiveKind::BezierCurve.is_parametric());
        assert!(PrimitiveKind::Circle.is_analytical());
        assert!(PrimitiveKind::TriangleMesh.is_mesh());
    }

    // Geometry utils tests
    #[test]
    fn test_point2d_conversion() {
        let point = Point2D::new(1.5, 2.5);
        let (x, y) = point2d_to_f64(&point);
        assert_eq!(x, 1.5);
        assert_eq!(y, 2.5);

        let restored = point2d_from_f64(x, y);
        assert_eq!(restored.x(), 1.5);
        assert_eq!(restored.y(), 2.5);
    }

    #[test]
    fn test_point3d_conversion() {
        let point = Point3D::new(1.5, 2.5, 3.5);
        let (x, y, z) = point3d_to_f64(&point);
        assert_eq!(x, 1.5);
        assert_eq!(y, 2.5);
        assert_eq!(z, 3.5);

        let restored = point3d_from_f64(x, y, z);
        assert_eq!(restored.x(), 1.5);
        assert_eq!(restored.y(), 2.5);
        assert_eq!(restored.z(), 3.5);
    }

    #[test]
    fn test_bounding_box_2d() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 1.0),
            Point2D::new(1.0, 3.0),
        ];

        let bbox = point2d_bounding_box(&points).unwrap();
        assert_eq!(bbox, (0.0, 0.0, 2.0, 3.0));
    }

    #[test]
    fn test_centroid_2d() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(2.0, 0.0),
            Point2D::new(1.0, 2.0),
        ];

        let centroid = point2d_centroid(&points).unwrap();
        assert_eq!(centroid.x(), 1.0);
        assert_eq!(centroid.y(), 2.0 / 3.0);
    }

    // Primitive trait tests
    struct MockPrimitive {
        kind: PrimitiveKind,
        bbox: BBox3D,
        measure: Option<f64>,
    }

    impl GeometricPrimitive for MockPrimitive {
        fn primitive_kind(&self) -> PrimitiveKind {
            self.kind
        }

        fn bounding_box(&self) -> BBox3D {
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
            bbox: BBox3D::new((0.0, 0.0, 0.0), (1.0, 1.0, 1.0)),
            measure: Some(5.0),
        };

        assert_eq!(primitive.primitive_kind(), PrimitiveKind::Point);
        assert_eq!(primitive.measure(), Some(5.0));

        let bbox = primitive.bounding_box();
        assert_eq!(bbox.min, Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max, Point3D::new(1.0, 1.0, 1.0));
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

    // BBox trait tests
    // テスト用のモック実装（テスト専用）
    #[derive(Debug, Clone, PartialEq)]
    struct MockBBox<const D: usize> {
        min: [f64; D],
        max: [f64; D],
    }

    impl<const D: usize> BoundingBox<D> for MockBBox<D> {
        type Coord = f64;

        fn min(&self) -> [Self::Coord; D] {
            self.min
        }

        fn max(&self) -> [Self::Coord; D] {
            self.max
        }

        fn new(min: [Self::Coord; D], max: [Self::Coord; D]) -> Self {
            Self { min, max }
        }

        fn extent(&self, dim: usize) -> Self::Coord {
            if dim < D {
                self.max[dim] - self.min[dim]
            } else {
                0.0
            }
        }

        fn volume(&self) -> Self::Coord {
            let mut vol = 1.0;
            for i in 0..D {
                vol *= self.extent(i);
            }
            vol
        }

        fn center(&self) -> [Self::Coord; D] {
            let mut center = [0.0; D];
            for i in 0..D {
                center[i] = (self.min[i] + self.max[i]) / 2.0;
            }
            center
        }
    }

    impl<const D: usize> BoundingBoxOps<D> for MockBBox<D> {
        fn contains_point(&self, point: [Self::Coord; D]) -> bool {
            for i in 0..D {
                if point[i] < self.min[i] || point[i] > self.max[i] {
                    return false;
                }
            }
            true
        }

        fn intersects(&self, other: &Self) -> bool {
            for i in 0..D {
                if self.max[i] < other.min[i] || self.min[i] > other.max[i] {
                    return false;
                }
            }
            true
        }

        fn union(&self, other: &Self) -> Self {
            let mut min = [0.0; D];
            let mut max = [0.0; D];
            for i in 0..D {
                min[i] = self.min[i].min(other.min[i]);
                max[i] = self.max[i].max(other.max[i]);
            }
            Self::new(min, max)
        }

        fn expand(&self, amount: Self::Coord) -> Self {
            let mut min = self.min;
            let mut max = self.max;
            for i in 0..D {
                min[i] -= amount;
                max[i] += amount;
            }
            Self::new(min, max)
        }

        fn is_valid(&self) -> bool {
            for i in 0..D {
                if self.min[i] > self.max[i] {
                    return false;
                }
            }
            true
        }
    }

    #[test]
    fn test_generic_bbox_2d() {
        let bbox = MockBBox::<2>::new([0.0, 0.0], [2.0, 3.0]);

        assert_eq!(bbox.min(), [0.0, 0.0]);
        assert_eq!(bbox.max(), [2.0, 3.0]);
        assert_eq!(bbox.extent(0), 2.0);
        assert_eq!(bbox.extent(1), 3.0);
        assert_eq!(bbox.volume(), 6.0);
        assert_eq!(bbox.center(), [1.0, 1.5]);
        assert!(bbox.is_valid());
    }

    #[test]
    fn test_generic_bbox_3d() {
        let bbox = MockBBox::<3>::new([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);

        assert_eq!(bbox.volume(), 24.0);
        assert_eq!(bbox.center(), [1.0, 1.5, 2.0]);
        assert!(bbox.contains_point([1.0, 1.0, 1.0]));
        assert!(!bbox.contains_point([3.0, 1.0, 1.0]));
    }

    #[test]
    fn test_bbox_operations() {
        let bbox1 = MockBBox::<3>::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let bbox2 = MockBBox::<3>::new([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        let bbox3 = MockBBox::<3>::new([3.0, 3.0, 3.0], [4.0, 4.0, 4.0]);

        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));

        let union = bbox1.union(&bbox2);
        assert_eq!(union.min(), [0.0, 0.0, 0.0]);
        assert_eq!(union.max(), [3.0, 3.0, 3.0]);

        let expanded = bbox1.expand(0.5);
        assert_eq!(expanded.min(), [-0.5, -0.5, -0.5]);
        assert_eq!(expanded.max(), [2.5, 2.5, 2.5]);
    }
}
