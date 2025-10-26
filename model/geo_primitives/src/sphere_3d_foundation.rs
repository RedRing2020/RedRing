//! Sphere3D Foundation Trait Implementations
//!
//! geo_foundation トレイトの実装

use crate::{BBox3D, Sphere3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

impl<T: Scalar> ExtensionFoundation<T> for Sphere3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Sphere
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }

    fn measure(&self) -> Option<T> {
        Some(self.volume())
    }
}

impl<T: Scalar> TolerantEq<T> for Sphere3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の距離と半径の差を直接計算で比較
        let center_distance = self.center().distance_to(&other.center());
        let radius_diff = (self.radius() - other.radius()).abs();

        // 許容誤差として単一のスカラー値を使用
        center_distance <= tolerance && radius_diff <= tolerance
    }
}

// std::cmp::PartialEq の実装は既に derive で提供されている

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_extension_foundation() {
        let sphere = Sphere3D::new(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_eq!(sphere.primitive_kind(), PrimitiveKind::Sphere);
        assert!(sphere.measure().is_some());

        let bbox = sphere.bounding_box();
        assert_eq!(bbox.min().x(), -4.0);
        assert_eq!(bbox.max().x(), 6.0);
    }

    #[test]
    fn test_tolerant_eq() {
        let sphere1 = Sphere3D::new(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();
        let sphere2 = Sphere3D::new(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();
        let sphere3 = Sphere3D::new(Point3D::new(1.0, 0.0, 0.0), 5.0).unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(sphere1.tolerant_eq(&sphere2, tolerance));
        assert!(!sphere1.tolerant_eq(&sphere3, tolerance));
    }
}
