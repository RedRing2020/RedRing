//! Vector3D の Foundation トレイト実装

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Vector3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Vector
    }

    fn bounding_box(&self) -> Self::BBox {
        // ベクトルの境界ボックスは原点から終点まで
        let _origin: Point3D<T> = Point3D::origin();
        let _endpoint = Point3D::new(self.x(), self.y(), self.z());

        let min_x = T::ZERO.min(self.x());
        let max_x = T::ZERO.max(self.x());
        let min_y = T::ZERO.min(self.y());
        let max_y = T::ZERO.max(self.y());
        let min_z = T::ZERO.min(self.z());
        let max_z = T::ZERO.max(self.z());

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    fn measure(&self) -> Option<T> {
        // ベクトルの測度は長さ
        Some(self.magnitude())
    }
}

impl<T: Scalar> TolerantEq<T> for Vector3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // ベクトルの差の大きさを計算
        let diff = *self - *other;
        let diff_magnitude = diff.magnitude();

        // 許容誤差として単一のスカラー値を使用
        diff_magnitude <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_foundation() {
        let vector = Vector3D::new(3.0, 4.0, 0.0);

        assert_eq!(vector.primitive_kind(), PrimitiveKind::Vector);
        assert!(vector.measure().is_some());
        assert_eq!(vector.measure().unwrap(), vector.magnitude());

        let bbox = vector.bounding_box();
        assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max(), Point3D::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn test_tolerant_eq() {
        let vector1 = Vector3D::new(1.0, 2.0, 3.0);
        let vector2 = Vector3D::new(1.0, 2.0, 3.0);
        let vector3 = Vector3D::new(2.0, 3.0, 4.0);

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(vector1.tolerant_eq(&vector2, tolerance));
        assert!(!vector1.tolerant_eq(&vector3, tolerance));
    }
}
