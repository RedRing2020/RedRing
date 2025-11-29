//! Vector3D の Foundation トレイト実装

use crate::Vector3D;
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Vector3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Vector
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

        // Vectors don't have bounding boxes in the new architecture
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
