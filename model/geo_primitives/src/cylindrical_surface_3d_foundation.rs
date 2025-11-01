//! CylindricalSurface3D の Foundation Pattern 実装
//!
//! ExtensionFoundation と TolerantEq トレイトの実装
//! ハイブリッドモデラーの分類システムとの統合

use crate::{BBox3D, CylindricalSurface3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar, TolerantEq};

// ============================================================================
// ExtensionFoundation Implementation
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for CylindricalSurface3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::CylindricalSurface
    }

    fn bounding_box(&self) -> Self::BBox {
        // サーフェスは無限軸方向のため、径方向の境界のみ
        // 実際の用途では境界制約が必要
        self.bounding_box_radial()
    }

    fn measure(&self) -> Option<T> {
        // サーフェスの測度は面積だが、無限サーフェスのため None
        // 境界制約された場合のみ有限の面積を持つ
        None
    }
}

// ============================================================================
// TolerantEq Implementation
// ============================================================================

impl<T: Scalar> TolerantEq<T> for CylindricalSurface3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の比較
        let center_diff = self.center().to_vector() - other.center().to_vector();
        if center_diff.length() > tolerance {
            return false;
        }

        // 軸の比較（方向は逆でも同じ軸）
        let axis_diff = self.axis().as_vector() - other.axis().as_vector();
        if axis_diff.length() > tolerance {
            // 逆方向もチェック
            let axis_diff_reversed = self.axis().as_vector() + other.axis().as_vector();
            if axis_diff_reversed.length() > tolerance {
                return false;
            }
        }

        // 参照方向の比較
        let ref_diff = self.ref_direction().as_vector() - other.ref_direction().as_vector();
        if ref_diff.length() > tolerance {
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

        let bbox = surface.bounding_box();
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
