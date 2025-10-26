//! Arc3D の Foundation トレイト実装

use crate::{Arc3D, BBox3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Arc3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Arc
    }

    fn bounding_box(&self) -> Self::BBox {
        // 円弧の開始点と終了点を含む境界ボックスを計算
        let _start_point = self.start_point();
        let _end_point = self.end_point();

        // 円弧の中心と半径から包含する境界ボックスを計算
        let center = self.center();
        let radius = self.radius();

        // 単純化のため、円全体の境界ボックスを返す
        // 実際の実装では角度範囲を考慮する必要がある
        let min_point = crate::Point3D::new(
            center.x() - radius,
            center.y() - radius,
            center.z() - radius,
        );
        let max_point = crate::Point3D::new(
            center.x() + radius,
            center.y() + radius,
            center.z() + radius,
        );

        BBox3D::new(min_point, max_point)
    }

    fn measure(&self) -> Option<T> {
        Some(self.arc_length())
    }
}

impl<T: Scalar> TolerantEq<T> for Arc3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点、半径、開始角、終了角の比較
        let center_distance = self.center().distance_to(&other.center());
        let radius_diff = (self.radius() - other.radius()).abs();
        let start_angle_diff = (self.start_angle() - other.start_angle())
            .to_radians()
            .abs();
        let end_angle_diff = (self.end_angle() - other.end_angle()).to_radians().abs();

        // 許容誤差として単一のスカラー値を使用
        let tolerance_angle = T::from_f64(0.01); // 約0.57度

        center_distance <= tolerance
            && radius_diff <= tolerance
            && start_angle_diff <= tolerance_angle
            && end_angle_diff <= tolerance_angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Point3D, Vector3D};

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let start_angle = geo_foundation::Angle::from_radians(0.0);
        let end_angle = geo_foundation::Angle::from_radians(std::f64::consts::PI);
        let arc = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();

        assert_eq!(arc.primitive_kind(), PrimitiveKind::Arc);
        assert!(arc.measure().is_some());
        assert_eq!(arc.measure().unwrap(), arc.arc_length());

        let bbox = arc.bounding_box();
        // 中心を含む境界ボックス
        assert!(bbox.min().x() <= center.x() && center.x() <= bbox.max().x());
    }

    #[test]
    fn test_tolerant_eq() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_dir = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let start_angle = geo_foundation::Angle::from_radians(0.0);
        let end_angle = geo_foundation::Angle::from_radians(std::f64::consts::PI);

        let arc1 = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();
        let arc2 = Arc3D::new(center, 5.0, normal, start_dir, start_angle, end_angle).unwrap();
        let arc3 = Arc3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            5.0,
            normal,
            start_dir,
            start_angle,
            end_angle,
        )
        .unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(arc1.tolerant_eq(&arc2, tolerance));
        assert!(!arc1.tolerant_eq(&arc3, tolerance));
    }
}
