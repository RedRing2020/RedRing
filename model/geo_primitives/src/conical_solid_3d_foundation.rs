//! ConicalSolid3D Foundation Implementation
//!
//! ExtensionFoundation トレイトによる統一インターフェースの実装
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::ConicalSolid3D;
use geo_contracts::{
    Bounded, MeasureFoundation, PrimitiveKind, PrimitiveMetadata, Scalar, TolerantEq,
};
use geo_core::Aabb3D;

impl<T: Scalar> PrimitiveMetadata for ConicalSolid3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::ConicalSolid
    }
}

impl<T: Scalar> MeasureFoundation<T> for ConicalSolid3D<T> {
    fn measure(&self) -> Option<T> {
        Some(self.volume_internal())
    }
}

impl<T: Scalar> Bounded<T> for ConicalSolid3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        Some(self.bounding_box())
    }
}

impl<T: Scalar> TolerantEq<T> for ConicalSolid3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の比較
        let center_self = self.center_internal();
        let center_other = other.center_internal();
        let center_diff = (center_self.x() - center_other.x()).abs()
            + (center_self.y() - center_other.y()).abs()
            + (center_self.z() - center_other.z()).abs();

        // 軸方向の比較（正規化済みベクトル）
        let axis_self = self.axis_internal();
        let axis_other = other.axis_internal();
        let axis_diff = (axis_self.x() - axis_other.x()).abs()
            + (axis_self.y() - axis_other.y()).abs()
            + (axis_self.z() - axis_other.z()).abs();

        // 参照方向の比較（正規化済みベクトル）
        let ref_self = self.ref_direction_internal();
        let ref_other = other.ref_direction_internal();
        let ref_diff = (ref_self.x() - ref_other.x()).abs()
            + (ref_self.y() - ref_other.y()).abs()
            + (ref_self.z() - ref_other.z()).abs();

        // 半径と高さの比較
        let radius_diff = (self.radius_internal() - other.radius_internal()).abs();
        let height_diff = (self.height_internal() - other.height_internal()).abs();

        center_diff <= tolerance
            && axis_diff <= tolerance
            && ref_diff <= tolerance
            && radius_diff <= tolerance
            && height_diff <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};
    use approx::assert_relative_eq;

    #[test]
    fn test_extension_foundation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let conical_solid = ConicalSolid3D::new(center, axis, ref_direction, 5.0, 10.0).unwrap();

        // primitive_kind のテスト
        assert_eq!(conical_solid.primitive_kind(), PrimitiveKind::ConicalSolid);

        // measure (体積) のテスト
        assert!(conical_solid.measure().is_some());
        let volume = conical_solid.measure().unwrap();
        let expected_volume = std::f64::consts::PI * 25.0 * 10.0 / 3.0; // π * r² * h / 3
        assert_relative_eq!(volume, expected_volume, epsilon = 1e-10);

        // bounding_box のテスト
        let bbox = conical_solid.aabb().expect("should have aabb");
        // 底面: center ± radius in x,y directions
        // 頂点: center + axis * height
        assert_relative_eq!(bbox.min().x(), -4.0, epsilon = 1e-10); // 1 - 5
        assert_relative_eq!(bbox.max().x(), 6.0, epsilon = 1e-10); // 1 + 5
        assert_relative_eq!(bbox.min().y(), -3.0, epsilon = 1e-10); // 2 - 5
        assert_relative_eq!(bbox.max().y(), 7.0, epsilon = 1e-10); // 2 + 5
        assert_relative_eq!(bbox.min().z(), 3.0, epsilon = 1e-10); // min(3, 13) = 3
        assert_relative_eq!(bbox.max().z(), 13.0, epsilon = 1e-10); // max(3, 13) = 13
    }

    #[test]
    fn test_tolerant_eq() {
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let conical_solid1 = ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            ref_direction,
            5.0,
            10.0,
        )
        .unwrap();

        let conical_solid2 = ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            ref_direction,
            5.0,
            10.0,
        )
        .unwrap();

        let tolerance = 0.01;

        // 自己比較
        assert!(conical_solid1.tolerant_eq(&conical_solid1, tolerance));

        // 同一円錐の比較
        assert!(conical_solid1.tolerant_eq(&conical_solid2, tolerance));

        // 対称性
        assert_eq!(
            conical_solid1.tolerant_eq(&conical_solid2, tolerance),
            conical_solid2.tolerant_eq(&conical_solid1, tolerance)
        );
    }

    #[test]
    fn test_tolerant_eq_different_cones() {
        let ref_direction = Vector3D::new(1.0, 0.0, 0.0);
        let conical_solid1 = ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            ref_direction,
            5.0,
            10.0,
        )
        .unwrap();

        let conical_solid2 = ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            ref_direction,
            3.0, // 異なる半径
            10.0,
        )
        .unwrap();

        let tolerance = 0.01;

        // 異なる円錐は等しくない
        assert!(!conical_solid1.tolerant_eq(&conical_solid2, tolerance));
    }

    #[test]
    fn test_conical_solid_degenerate() {
        // ゼロ半径の円錐
        let zero_radius = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 0.0, 5.0);
        assert!(zero_radius.is_none());

        // ゼロ高さの円錐
        let zero_height = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, 0.0);
        assert!(zero_height.is_none());

        // 負の半径
        let negative_radius = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), -1.0, 5.0);
        assert!(negative_radius.is_none());

        // 負の高さ
        let negative_height = ConicalSolid3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0, -1.0);
        assert!(negative_height.is_none());
    }
}
