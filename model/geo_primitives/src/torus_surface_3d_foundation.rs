// torus_surface_3d_foundation.rs
// TorusSurface3D の ExtensionFoundation トレイト実装
//
// Foundation パターンに従い、統一されたプリミティブインターフェースを提供します。

use crate::{BBox3D, Point3D, TorusSurface3D};
use geo_foundation::{ExtensionFoundation, PrimitiveKind, Scalar};

impl<T: Scalar> ExtensionFoundation<T> for TorusSurface3D<T> {
    type BBox = BBox3D<T>;

    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::TorusSurface
    }

    /// 境界ボックスを計算
    ///
    /// トーラス面の境界ボックスは軸配置を考慮して計算されます。
    /// 3D CAM での工具パス計算において重要な情報です。
    fn bounding_box(&self) -> Self::BBox {
        // トーラスの外半径（最大半径）
        let outer_radius = self.major_radius() + self.minor_radius();

        // 軸方向の範囲
        let z_extent = self.minor_radius();

        // 局所座標系での境界を計算
        let origin = self.origin();

        // Z軸方向の成分を考慮した境界計算
        let z_axis = self.z_axis();
        let z_vec_x = z_axis.x() * z_extent;
        let z_vec_y = z_axis.y() * z_extent;
        let z_vec_z = z_axis.z() * z_extent;

        // X, Y軸方向の最大範囲を計算
        let x_axis = self.x_axis();
        let y_axis = self.y_axis();

        // 各軸成分の最大値を計算
        let x_max_from_x = x_axis.x().abs() * outer_radius;
        let x_max_from_y = y_axis.x().abs() * outer_radius;
        let x_max_from_z = z_vec_x.abs();
        let x_extent = x_max_from_x + x_max_from_y + x_max_from_z;

        let y_max_from_x = x_axis.y().abs() * outer_radius;
        let y_max_from_y = y_axis.y().abs() * outer_radius;
        let y_max_from_z = z_vec_y.abs();
        let y_extent = y_max_from_x + y_max_from_y + y_max_from_z;

        let z_max_from_x = x_axis.z().abs() * outer_radius;
        let z_max_from_y = y_axis.z().abs() * outer_radius;
        let z_max_from_z = z_vec_z.abs();
        let z_extent_final = z_max_from_x + z_max_from_y + z_max_from_z;

        // 境界ボックスの最小・最大点
        let min_point = Point3D::new(
            origin.x() - x_extent,
            origin.y() - y_extent,
            origin.z() - z_extent_final,
        );

        let max_point = Point3D::new(
            origin.x() + x_extent,
            origin.y() + y_extent,
            origin.z() + z_extent_final,
        );

        BBox3D::new(min_point, max_point)
    }

    /// 測度（表面積）を計算
    ///
    /// トーラス面の表面積: 4π² × major_radius × minor_radius
    /// 3D CAM での材料除去量計算に使用されます。
    fn measure(&self) -> Option<T> {
        Some(self.surface_area())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Vector3D};
    use std::f64::consts::PI;

    #[test]
    fn test_primitive_kind() {
        let torus = TorusSurface3D::standard(3.0, 1.0).unwrap();
        assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSurface);
    }

    #[test]
    fn test_surface_area_measure() {
        let major_radius = 3.0;
        let minor_radius = 1.0;
        let torus = TorusSurface3D::standard(major_radius, minor_radius).unwrap();

        let measure = torus.measure().unwrap();
        let expected = 4.0 * PI * PI * major_radius * minor_radius;

        assert!((measure - expected).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_box_standard_torus() {
        let major_radius = 3.0;
        let minor_radius = 1.0;
        let torus = TorusSurface3D::standard(major_radius, minor_radius).unwrap();

        let bbox = torus.bounding_box();
        let outer_radius = major_radius + minor_radius; // 4.0

        // 標準トーラス（XY平面、Z軸中心）の境界ボックス
        assert!((bbox.min().x() + outer_radius).abs() < 1e-10);
        assert!((bbox.max().x() - outer_radius).abs() < 1e-10);
        assert!((bbox.min().y() + outer_radius).abs() < 1e-10);
        assert!((bbox.max().y() - outer_radius).abs() < 1e-10);
        assert!((bbox.min().z() + minor_radius).abs() < 1e-10);
        assert!((bbox.max().z() - minor_radius).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_box_rotated_torus() {
        let major_radius = 2.0;
        let minor_radius = 0.5;

        // Y軸回転したトーラス（Z軸がX方向）
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let z_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(); // X方向
        let x_axis = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(); // Z方向

        let torus =
            TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap();
        let bbox = torus.bounding_box();

        let outer_radius = major_radius + minor_radius; // 2.5

        // 回転後の境界ボックスをチェック
        // X方向（元のZ軸方向）に拡張
        assert!((bbox.min().x() - (origin.x() - minor_radius)).abs() < 1e-10);
        assert!((bbox.max().x() - (origin.x() + minor_radius)).abs() < 1e-10);

        // Y方向は変化なし
        assert!((bbox.min().y() - (origin.y() - outer_radius)).abs() < 1e-10);
        assert!((bbox.max().y() - (origin.y() + outer_radius)).abs() < 1e-10);

        // Z方向（元のX軸方向）に拡張
        assert!((bbox.min().z() - (origin.z() - outer_radius)).abs() < 1e-10);
        assert!((bbox.max().z() - (origin.z() + outer_radius)).abs() < 1e-10);
    }

    #[test]
    fn test_cam_relevant_properties() {
        // CAM での工具オフセット計算で重要な特性をテスト
        let torus = TorusSurface3D::donut(5.0, 1.0).unwrap(); // ドーナツ型

        // 表面積が正の値
        assert!(torus.measure().unwrap() > 0.0);

        // 境界ボックスが有効
        let bbox = torus.bounding_box();
        assert!(bbox.min().x() < bbox.max().x());
        assert!(bbox.min().y() < bbox.max().y());
        assert!(bbox.min().z() < bbox.max().z());

        // プリミティブ種類が正しい
        assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSurface);
    }
}
