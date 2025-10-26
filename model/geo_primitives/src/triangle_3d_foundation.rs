//! Triangle3D の Foundation トレイト実装

use crate::{BBox3D, Triangle3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Triangle3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Triangle
    }

    fn bounding_box(&self) -> Self::BBox {
        // 3つの頂点の最小/最大座標を計算
        let vertices = [self.vertex_a(), self.vertex_b(), self.vertex_c()];

        let min_x = vertices
            .iter()
            .map(|v| v.x())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_x = vertices
            .iter()
            .map(|v| v.x())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_y = vertices
            .iter()
            .map(|v| v.y())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_y = vertices
            .iter()
            .map(|v| v.y())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_z = vertices
            .iter()
            .map(|v| v.z())
            .fold(T::INFINITY, |a, b| a.min(b));
        let max_z = vertices
            .iter()
            .map(|v| v.z())
            .fold(-T::INFINITY, |a, b| a.max(b));

        let min_point = crate::Point3D::new(min_x, min_y, min_z);
        let max_point = crate::Point3D::new(max_x, max_y, max_z);

        BBox3D::new(min_point, max_point)
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}

impl<T: Scalar> TolerantEq<T> for Triangle3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 3つの頂点がそれぞれ許容誤差内にあるかチェック
        let a_distance = self.vertex_a().distance_to(&other.vertex_a());
        let b_distance = self.vertex_b().distance_to(&other.vertex_b());
        let c_distance = self.vertex_c().distance_to(&other.vertex_c());

        a_distance <= tolerance && b_distance <= tolerance && c_distance <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_extension_foundation() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        assert_eq!(triangle.primitive_kind(), PrimitiveKind::Triangle);
        assert!(triangle.measure().is_some());
        assert_eq!(triangle.measure().unwrap(), triangle.area());

        let bbox = triangle.bounding_box();
        assert_eq!(bbox.min().x(), 0.0);
        assert_eq!(bbox.max().x(), 1.0);
        assert_eq!(bbox.min().y(), 0.0);
        assert_eq!(bbox.max().y(), 1.0);
    }

    #[test]
    fn test_tolerant_eq() {
        let triangle1 = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let triangle2 = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let triangle3 = Triangle3D::new(
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(2.0, 1.0, 1.0),
            Point3D::new(1.0, 2.0, 1.0),
        )
        .unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(triangle1.tolerant_eq(&triangle2, tolerance));
        assert!(!triangle1.tolerant_eq(&triangle3, tolerance));
    }
}
