//! Circle3D の Foundation トレイト実装

use crate::{BBox3D, Circle3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for Circle3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }

    fn bounding_box(&self) -> Self::BBox {
        // 円の包含する境界ボックスを計算
        // 法線ベクトルに垂直な2つのベクトルを作成
        let normal = self.normal().as_vector();

        // 法線に垂直な任意のベクトルを作成
        let temp = if normal.x().abs() > T::from_f64(0.1) {
            crate::Vector3D::new(T::ZERO, T::ONE, T::ZERO)
        } else {
            crate::Vector3D::new(T::ONE, T::ZERO, T::ZERO)
        };

        let u = normal.cross(&temp).normalize();
        let v = normal.cross(&u).normalize();

        // 円周上の点の範囲を計算
        let center = self.center();
        let radius = self.radius();

        let u_extent = u * radius;
        let v_extent = v * radius;

        // X, Y, Z 方向の最大/最小値を計算
        let u_x = u_extent.x().abs();
        let u_y = u_extent.y().abs();
        let u_z = u_extent.z().abs();

        let v_x = v_extent.x().abs();
        let v_y = v_extent.y().abs();
        let v_z = v_extent.z().abs();

        let extent_x = u_x + v_x;
        let extent_y = u_y + v_y;
        let extent_z = u_z + v_z;

        let min_point = crate::Point3D::new(
            center.x() - extent_x,
            center.y() - extent_y,
            center.z() - extent_z,
        );
        let max_point = crate::Point3D::new(
            center.x() + extent_x,
            center.y() + extent_y,
            center.z() + extent_z,
        );

        BBox3D::new(min_point, max_point)
    }

    fn measure(&self) -> Option<T> {
        Some(self.area())
    }
}

impl<T: Scalar> TolerantEq<T> for Circle3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の距離を計算
        let center_distance = self.center().distance_to(&other.center());
        let radius_diff = (self.radius() - other.radius()).abs();

        // 法線ベクトルの類似性を検証（内積で角度を比較）
        let normal_dot = self
            .normal()
            .as_vector()
            .dot(&other.normal().as_vector())
            .abs();
        let normal_similar = normal_dot >= T::from_f64(0.999); // 約2.5度以内

        // 許容誤差として単一のスカラー値を使用
        center_distance <= tolerance && radius_diff <= tolerance && normal_similar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Point3D, Vector3D};

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let circle = Circle3D::new(center, normal, 5.0).unwrap();

        assert_eq!(circle.primitive_kind(), PrimitiveKind::Circle);
        assert!(circle.measure().is_some());
        assert_eq!(circle.measure().unwrap(), circle.area());

        let bbox = circle.bounding_box();
        // XY平面の円なので、Z方向の範囲は0
        assert!((bbox.min().z() - center.z()).abs() < 1e-10);
        assert!((bbox.max().z() - center.z()).abs() < 1e-10);
    }

    #[test]
    fn test_tolerant_eq() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();

        let circle1 = Circle3D::new(center, normal, 5.0).unwrap();
        let circle2 = Circle3D::new(center, normal, 5.0).unwrap();
        let circle3 = Circle3D::new(Point3D::new(2.0, 0.0, 0.0), normal, 5.0).unwrap();

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(circle1.tolerant_eq(&circle2, tolerance));
        assert!(!circle1.tolerant_eq(&circle3, tolerance));
    }
}
