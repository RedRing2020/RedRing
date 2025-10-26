//! Ray3D の Foundation トレイト実装

use crate::{BBox3D, Point3D, Ray3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Ray3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ray
    }

    fn bounding_box(&self) -> Self::BBox {
        // 無限レイの境界ボックスは無限大
        // 実際には起点のみの境界ボックスを返す（レイの方向は無限）
        let origin = self.origin();

        // 方向に少し拡張した境界ボックスを作成
        let direction = self.direction().as_vector();
        let far_point = Point3D::new(
            origin.x() + direction.x() * T::from_f64(1000.0),
            origin.y() + direction.y() * T::from_f64(1000.0),
            origin.z() + direction.z() * T::from_f64(1000.0),
        );

        let min_x = origin.x().min(far_point.x());
        let max_x = origin.x().max(far_point.x());
        let min_y = origin.y().min(far_point.y());
        let max_y = origin.y().max(far_point.y());
        let min_z = origin.z().min(far_point.z());
        let max_z = origin.z().max(far_point.z());

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    fn measure(&self) -> Option<T> {
        // レイの測度は無限大（長さがない）
        None // 無限大なので None を返す
    }
}

impl<T: Scalar> TolerantEq<T> for Ray3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 起点の距離をチェック
        let origin_distance = self.origin().distance_to(&other.origin());

        // 方向ベクトルの類似性をチェック（内積）
        let direction_dot = self
            .direction()
            .as_vector()
            .dot(&other.direction().as_vector())
            .abs();
        let direction_similar = direction_dot >= T::from_f64(0.999); // 約2.5度以内

        // 許容誤差として単一のスカラー値を使用
        origin_distance <= tolerance && direction_similar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Vector3D};

    #[test]
    fn test_extension_foundation() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray = Ray3D::new(origin, direction.as_vector()).unwrap();

        assert_eq!(ray.primitive_kind(), PrimitiveKind::Ray);
        assert!(ray.measure().is_none()); // 無限大

        let _bbox = ray.bounding_box();
        // 無限の境界ボックスのテスト（実装に依存）
    }

    #[test]
    fn test_tolerant_eq() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let ray1 = Ray3D::new(origin, direction.as_vector()).unwrap();
        let ray2 = Ray3D::new(origin, direction.as_vector()).unwrap();
        let ray3 = Ray3D::new(Point3D::new(1.0, 0.0, 0.0), direction.as_vector()).unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(ray1.tolerant_eq(&ray2, tolerance));
        assert!(!ray1.tolerant_eq(&ray3, tolerance));
    }
}
