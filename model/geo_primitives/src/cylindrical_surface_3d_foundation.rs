//! CylindricalSurface3D の Foundation Pattern 実装
//!
//! ExtensionFoundation と TolerantEq トレイトの実装
//! ハイブリッドモデラーの分類システムとの統合

use crate::CylindricalSurface3D;
use geo_contracts::{Bounded, ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq};
use geo_core::Aabb3D;

// ============================================================================
// ExtensionFoundation Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for CylindricalSurface3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::CylindricalSurface
    }

    fn measure(&self) -> Option<T> {
        // サーフェスの測度は面積だが、無限サーフェスのため None
        // 境界制約された場合のみ有限の面積を持つ
        None
    }
}

impl<T: Scalar> Bounded<T> for CylindricalSurface3D<T> {
    type Aabb = Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // サーフェスは無限軸方向のため、径方向の境界のみ
        // 実際の用途では境界制約が必要
        Some(self.bounding_box_radial())
    }
}

// ============================================================================
// TolerantEq Implementation
// ============================================================================

impl<T: Scalar> TolerantEq<T> for CylindricalSurface3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の比較
        let dx = self.center_internal().x() - other.center_internal().x();
        let dy = self.center_internal().y() - other.center_internal().y();
        let dz = self.center_internal().z() - other.center_internal().z();
        let center_dist_sq = dx * dx + dy * dy + dz * dz;
        if center_dist_sq > tolerance * tolerance {
            return false;
        }

        // 軸の比較（方向は逆でも同じ軸）
        let axis_dx = self.axis().x() - other.axis().x();
        let axis_dy = self.axis().y() - other.axis().y();
        let axis_dz = self.axis().z() - other.axis().z();
        let axis_dist_sq = axis_dx * axis_dx + axis_dy * axis_dy + axis_dz * axis_dz;
        if axis_dist_sq > tolerance * tolerance {
            // 逆方向もチェック
            let axis_dx_rev = self.axis().x() + other.axis().x();
            let axis_dy_rev = self.axis().y() + other.axis().y();
            let axis_dz_rev = self.axis().z() + other.axis().z();
            let axis_dist_sq_rev =
                axis_dx_rev * axis_dx_rev + axis_dy_rev * axis_dy_rev + axis_dz_rev * axis_dz_rev;
            if axis_dist_sq_rev > tolerance * tolerance {
                return false;
            }
        }

        // 参照方向の比較
        let ref_dx = self.ref_direction().x() - other.ref_direction().x();
        let ref_dy = self.ref_direction().y() - other.ref_direction().y();
        let ref_dz = self.ref_direction().z() - other.ref_direction().z();
        let ref_dist_sq = ref_dx * ref_dx + ref_dy * ref_dy + ref_dz * ref_dz;
        if ref_dist_sq > tolerance * tolerance {
            return false;
        }

        // 半径の比較
        (self.radius() - other.radius()).abs() <= tolerance
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_extension_foundation() {
        let surface = CylindricalSurface3D::new_z_axis(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

        assert_eq!(surface.primitive_kind(), PrimitiveKind::CylindricalSurface);

        let bbox = surface.aabb().expect("should have aabb");
        assert!(!bbox.is_empty());

        // 無限サーフェスのため測度は None
        assert_eq!(surface.measure(), None);
    }

    #[test]
    fn test_tolerant_eq() {
        let surface1 = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();

        let surface2 =
            CylindricalSurface3D::new_z_axis(Point3D::new(0.0001, 0.0001, 0.0001), 5.0001).unwrap();

        // 許容誤差内で等しい
        assert!(surface1.tolerant_eq(&surface2, 1e-3));

        // 許容誤差外で異なる - tolerant_eqの逆
        assert!(!surface1.tolerant_eq(&surface2, 1e-5));
    }

    #[test]
    fn test_tolerant_eq_different_surfaces() {
        let surface1 = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();

        let surface2 = CylindricalSurface3D::new_z_axis(Point3D::new(10.0, 0.0, 0.0), 5.0).unwrap();

        // 中心が大きく異なる - tolerant_eqの逆
        assert!(!surface1.tolerant_eq(&surface2, 1e-3));
        assert!(!surface1.tolerant_eq(&surface2, 1e-3));
    }
}
