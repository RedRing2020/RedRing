//! Plane3D の Foundation トレイト実装

use crate::{BBox3D, Plane3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Plane3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }

    fn bounding_box(&self) -> Self::BBox {
        // 無限平面は境界ボックスを持たない
        // 理論的には無限大の境界ボックスを返すべきだが、
        // 実用上は None を表現するため、原点の微小な境界ボックスを返す
        let origin: crate::Point3D<T> = crate::Point3D::origin();
        let epsilon = T::EPSILON;

        let min_point = crate::Point3D::new(
            origin.x() - epsilon,
            origin.y() - epsilon,
            origin.z() - epsilon,
        );
        let max_point = crate::Point3D::new(
            origin.x() + epsilon,
            origin.y() + epsilon,
            origin.z() + epsilon,
        );

        BBox3D::from_points(&[min_point, max_point]).expect("Failed to create bounding box")
    }

    fn measure(&self) -> Option<T> {
        // 無限平面の測度（面積）は無限大なので None を返す
        None
    }
}

impl<T: Scalar> TolerantEq<T> for Plane3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 平面の等価性を誤差許容で判定
        // 1. 法線ベクトルが同じ方向を向いているか
        let normal1 = self.normal();
        let normal2 = other.normal();

        // 法線の方向が同じまたは反対か（平行性チェック）
        let dot_product = normal1.dot(&normal2).abs();
        let one = T::ONE;
        if (dot_product - one).abs() > tolerance {
            return false;
        }

        // 2. 平面上の任意の点から他方の平面までの距離が許容範囲内か
        let point_on_plane1 = self.point();
        let distance = other.distance_to_point(point_on_plane1);

        distance <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};
    use geo_foundation::{ExtensionFoundation, TolerantEq};

    #[test]
    fn test_extension_foundation() {
        let point = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        // primitive_kind のテスト
        assert_eq!(plane.primitive_kind(), PrimitiveKind::Plane);

        // bounding_box のテスト（無限平面なので微小な境界ボックス）
        let bbox = plane.bounding_box();
        // 境界ボックスが存在することを確認
        assert!(bbox.width() > 0.0);
        assert!(bbox.height() > 0.0);
        assert!(bbox.depth() > 0.0);

        // measure のテスト（無限平面なので None）
        let measure = plane.measure();
        assert!(measure.is_none());
    }

    #[test]
    fn test_tolerant_eq() {
        let point1 = Point3D::new(0.0, 0.0, 0.0);
        let normal1 = Vector3D::new(0.0, 0.0, 1.0);
        let plane1 = Plane3D::from_point_and_normal(point1, normal1).unwrap();

        let point2 = Point3D::new(0.0, 0.0, 0.001); // わずかに異なる点
        let normal2 = Vector3D::new(0.0, 0.0, 1.0); // 同じ法線
        let plane2 = Plane3D::from_point_and_normal(point2, normal2).unwrap();

        let tolerance = 0.01;

        // 自己比較
        assert!(plane1.tolerant_eq(&plane1, tolerance));

        // わずかな差がある平面同士（許容範囲内）
        assert!(plane1.tolerant_eq(&plane2, tolerance));

        // 対称性
        assert_eq!(
            plane1.tolerant_eq(&plane2, tolerance),
            plane2.tolerant_eq(&plane1, tolerance)
        );
    }

    #[test]
    fn test_tolerant_eq_different_planes() {
        let point1 = Point3D::new(0.0, 0.0, 0.0);
        let normal1 = Vector3D::new(0.0, 0.0, 1.0);
        let plane1 = Plane3D::from_point_and_normal(point1, normal1).unwrap();

        let point2 = Point3D::new(0.0, 0.0, 0.0);
        let normal2 = Vector3D::new(1.0, 0.0, 0.0); // 全く異なる法線
        let plane2 = Plane3D::from_point_and_normal(point2, normal2).unwrap();

        let tolerance = 0.01;

        // 異なる平面は等しくない
        assert!(!plane1.tolerant_eq(&plane2, tolerance));
    }
}
