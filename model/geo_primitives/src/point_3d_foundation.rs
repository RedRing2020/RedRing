//! Point3D の Foundation トレイト実装

use crate::{BBox3D, Point3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Point3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Point
    }

    fn bounding_box(&self) -> Self::BBox {
        // 点の境界ボックスは点自身
        BBox3D::from_point(*self)
    }

    fn measure(&self) -> Option<T> {
        // 点の測度は0（次元がない）
        Some(T::ZERO)
    }
}

impl<T: Scalar> TolerantEq<T> for Point3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 2点間の距離を計算
        let distance = self.distance_to(other);

        // スカラー値の許容誤差と比較
        distance <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_foundation() {
        let point = Point3D::new(1.0, 2.0, 3.0);

        assert_eq!(point.primitive_kind(), PrimitiveKind::Point);
        assert!(point.measure().is_some());
        assert_eq!(point.measure().unwrap(), 0.0);

        let bbox = point.bounding_box();
        assert_eq!(bbox.min(), point);
        assert_eq!(bbox.max(), point);
    }

    #[test]
    fn test_tolerant_eq() {
        let point1 = Point3D::new(1.0, 2.0, 3.0);
        let point2 = Point3D::new(1.0, 2.0, 3.0);
        let point3 = Point3D::new(2.0, 3.0, 4.0);

        // point1とpoint3の距離は sqrt(3) ≈ 1.732

        // 小さい許容誤差（1mmレベル）
        let small_tolerance = 0.001;

        // 中程度の許容誤差（2mmレベル）
        let medium_tolerance = 0.002;

        // 大きい許容誤差（2mレベル）
        let large_tolerance = 2.0;

        // 同一点
        assert!(point1.tolerant_eq(&point2, small_tolerance));

        // 小さい許容誤差では等価ではない（距離1.732 > 0.001）
        assert!(!point1.tolerant_eq(&point3, small_tolerance));

        // 中程度の許容誤差でも等価ではない（距離1.732 > 0.002）
        assert!(!point1.tolerant_eq(&point3, medium_tolerance));

        // 大きい許容誤差では等価とみなされる（距離1.732 < 2.0）
        assert!(point1.tolerant_eq(&point3, large_tolerance));

        // より現実的なテスト：近い点での比較
        let close_point = Point3D::new(1.0005, 2.0005, 3.0005);
        let micro_tolerance = 0.001;

        // 微小な差（距離約0.00087）は許容誤差内で等価
        assert!(point1.tolerant_eq(&close_point, micro_tolerance));
    }
}
