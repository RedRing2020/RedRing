//! ConicalSolid3D Foundation Implementation
//!
//! ExtensionFoundation トレイトによる統一インターフェースの実装
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**

use crate::{BBox3D, ConicalSolid3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq};

impl<T: Scalar> ExtensionFoundation<T> for ConicalSolid3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Cone
    }

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }

    fn measure(&self) -> Option<T> {
        Some(self.volume())
    }
}

impl<T: Scalar> TolerantEq<T> for ConicalSolid3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の比較
        let center_diff = (self.center().x() - other.center().x()).abs()
            + (self.center().y() - other.center().y()).abs()
            + (self.center().z() - other.center().z()).abs();

        // 軸方向の比較（正規化済みベクトル）
        let axis_diff = (self.axis().x() - other.axis().x()).abs()
            + (self.axis().y() - other.axis().y()).abs()
            + (self.axis().z() - other.axis().z()).abs();

        // 参照方向の比較（正規化済みベクトル）
        let ref_diff = (self.ref_direction().x() - other.ref_direction().x()).abs()
            + (self.ref_direction().y() - other.ref_direction().y()).abs()
            + (self.ref_direction().z() - other.ref_direction().z()).abs();

        // 半径と高さの比較
        let radius_diff = (self.radius() - other.radius()).abs();
        let height_diff = (self.height() - other.height()).abs();

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
        assert_eq!(conical_solid.primitive_kind(), PrimitiveKind::Cone);

        // measure (体積) のテスト
        assert!(conical_solid.measure().is_some());
        let volume = conical_solid.measure().unwrap();
        let expected_volume = std::f64::consts::PI * 25.0 * 10.0 / 3.0; // π * r² * h / 3
        assert_relative_eq!(volume, expected_volume, epsilon = 1e-10);

        // bounding_box のテスト
        let bbox = conical_solid.bounding_box();
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
