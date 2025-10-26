//! BBox3D の Foundation トレイト実装

use crate::BBox3D;
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for BBox3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::BoundingBox
    }

    fn bounding_box(&self) -> Self::BBox {
        *self // 境界ボックス自身がその境界ボックス
    }

    fn measure(&self) -> Option<T> {
        Some(self.volume())
    }
}

impl<T: Scalar> TolerantEq<T> for BBox3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 最小点と最大点の距離をチェック
        let min_distance = self.min().distance_to(&other.min());
        let max_distance = self.max().distance_to(&other.max());

        // 許容誤差として単一のスカラー値を使用
        min_distance <= tolerance && max_distance <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_extension_foundation() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        assert_eq!(bbox.primitive_kind(), PrimitiveKind::BoundingBox);
        assert!(bbox.measure().is_some());
        assert_eq!(bbox.measure().unwrap(), bbox.volume());

        let self_bbox = bbox.bounding_box();
        assert_eq!(bbox, self_bbox);
    }

    #[test]
    fn test_tolerant_eq() {
        let bbox1 = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        let bbox2 = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        let bbox3 = BBox3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(11.0, 6.0, 4.0));

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(bbox1.tolerant_eq(&bbox2, tolerance));
        assert!(!bbox1.tolerant_eq(&bbox3, tolerance));
    }
}
