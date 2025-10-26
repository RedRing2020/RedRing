//! Cylinder3D の Foundation トレイト実装

use crate::{BBox3D, Cylinder3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Cylinder3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Cylinder
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }

    fn measure(&self) -> Option<T> {
        Some(self.volume())
    }
}

impl<T: Scalar> TolerantEq<T> for Cylinder3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の距離を計算
        let center_distance = self.center().distance_to(&other.center());
        let radius_diff = (self.radius() - other.radius()).abs();
        let height_diff = (self.height() - other.height()).abs();

        // 軸の方向の類似性を検証（内積で角度を比較）
        let axis_dot = self.axis().dot(&other.axis()).abs();
        let axis_similar = axis_dot >= T::from_f64(0.999); // 約2.5度以内

        // 許容誤差として単一のスカラー値を使用
        center_distance <= tolerance
            && radius_diff <= tolerance
            && height_diff <= tolerance
            && axis_similar
    }
}

// std::cmp::PartialEq の実装は既に derive で提供されている

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let cylinder = Cylinder3D::new(center, axis, 5.0, 10.0).unwrap();

        assert_eq!(cylinder.primitive_kind(), PrimitiveKind::Cylinder);
        assert!(cylinder.measure().is_some());

        let bbox = cylinder.bounding_box();
        assert_eq!(bbox.min().x(), -4.0);
        assert_eq!(bbox.max().x(), 6.0);
    }

    #[test]
    fn test_tolerant_eq() {
        let cylinder1 = Cylinder3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            5.0,
            10.0,
        )
        .unwrap();
        let cylinder2 = Cylinder3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            5.0,
            10.0,
        )
        .unwrap();
        let cylinder3 = Cylinder3D::new(
            Point3D::new(2.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            5.0,
            10.0,
        )
        .unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(cylinder1.tolerant_eq(&cylinder2, tolerance));
        assert!(!cylinder1.tolerant_eq(&cylinder3, tolerance));
    }
}
