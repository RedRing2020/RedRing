//! SphericalSolid3D Transform Operations
//!
//! STEP準拠球ソリッドの変換操作実装
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**
//!
//! ## 実装内容
//! - 平行移動：center の移動
//! - 回転：軸と参照方向の回転
//! - スケール：半径のスケーリング
//! - BasicTransform トレイト実装（Foundation パターン準拠）
//!
//! ## STEP準拠球ソリッド変換の特性
//! - 軸と参照方向の直交性保持：変換後も axis ⊥ ref_direction を維持
//! - 正規化保持：軸と参照方向が単位ベクトルのまま
//! - 右手系保持：Y軸 = Z軸 × X軸 関係を維持
//! - 幾何学的整合性：半径の正の値保持
//! - ソリッド特性保持：体積比例、内部判定整合性

use crate::{Point3D, SphericalSolid3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Basic Transform Operations
// ============================================================================

impl<T: Scalar> SphericalSolid3D<T> {
    /// 平行移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 平行移動後の球ソリッド
    ///
    /// # Note
    /// 中心のみが移動、軸・参照方向・半径は不変
    pub fn translate(&self, translation: Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );
        SphericalSolid3D::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
        )
        .expect("Translation should preserve valid sphere properties")
    }

    /// 均等スケール変換
    ///
    /// # Arguments
    /// * `scale_factor` - スケール係数（正の値）
    ///
    /// # Returns
    /// スケール変換後の球ソリッド、無効な係数の場合は None
    ///
    /// # Note
    /// 中心、軸方向、参照方向はそのまま、半径をスケール
    pub fn scale_uniform(&self, scale_factor: T) -> Option<Self> {
        if scale_factor <= T::ZERO {
            return None;
        }

        SphericalSolid3D::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius() * scale_factor,
        )
    }

    /// Z軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// Z軸周りに回転した球ソリッド
    ///
    /// # Note
    /// 原点を中心にZ軸周りに回転。center, axis, ref_direction が回転される
    pub fn rotate_z(&self, angle: &Angle<T>) -> Self {
        self.rotate_around_axis(
            &Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            &Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            angle,
        )
        .expect("Z-axis rotation should always be valid")
    }

    /// 任意軸周りの回転
    ///
    /// # Arguments
    /// * `rotation_center` - 回転中心点
    /// * `rotation_axis` - 回転軸（正規化される）
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// 回転後の球ソリッド、回転軸が無効な場合は None
    ///
    /// # Note
    /// center, axis, ref_direction を回転軸周りに回転
    /// 半径は不変（球の対称性により）
    pub fn rotate_around_axis(
        &self,
        rotation_center: &Point3D<T>,
        rotation_axis: &Vector3D<T>,
        angle: &Angle<T>,
    ) -> Option<Self> {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        // 回転軸の正規化
        let axis_norm = rotation_axis.magnitude();
        if axis_norm.is_zero() {
            return None;
        }
        let normalized_axis = *rotation_axis / axis_norm;

        // Rodrigues の回転公式のヘルパー関数
        let rotate_vector = |v: &Vector3D<T>| -> Vector3D<T> {
            let k = normalized_axis;
            let v_parallel = k * v.dot(&k);
            let v_perpendicular = *v - v_parallel;
            let w = k.cross(v);

            v_parallel + v_perpendicular * cos_theta + w * sin_theta
        };

        // 中心点の回転
        let center_to_rotation_center = Vector3D::new(
            self.center().x() - rotation_center.x(),
            self.center().y() - rotation_center.y(),
            self.center().z() - rotation_center.z(),
        );
        let rotated_center_offset = rotate_vector(&center_to_rotation_center);
        let new_center = Point3D::new(
            rotation_center.x() + rotated_center_offset.x(),
            rotation_center.y() + rotated_center_offset.y(),
            rotation_center.z() + rotated_center_offset.z(),
        );

        // 軸方向の回転
        let new_axis_vector = rotate_vector(&self.axis().as_vector());
        let new_axis = crate::Direction3D::from_vector(new_axis_vector)
            .expect("Rotated axis should remain valid");

        // 参照方向の回転
        let new_ref_direction_vector = rotate_vector(&self.ref_direction().as_vector());
        let new_ref_direction = crate::Direction3D::from_vector(new_ref_direction_vector)
            .expect("Rotated ref_direction should remain valid");

        SphericalSolid3D::new(
            new_center,
            new_axis.as_vector(),
            new_ref_direction.as_vector(),
            self.radius(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn create_test_spherical_solid() -> SphericalSolid3D<f64> {
        SphericalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 2.0).unwrap()
    }

    #[test]
    fn test_translate() {
        let spherical_solid = create_test_spherical_solid();
        let translation = Vector3D::new(5.0, -3.0, 2.0);

        let translated = spherical_solid.translate(translation);

        // 中心が移動
        assert_eq!(translated.center(), Point3D::new(6.0, -1.0, 5.0));

        // 他のプロパティは不変
        assert_eq!(translated.radius(), spherical_solid.radius());
        assert_eq!(translated.axis(), spherical_solid.axis());
        assert_eq!(translated.ref_direction(), spherical_solid.ref_direction());
    }

    #[test]
    fn test_scale_uniform() {
        let spherical_solid = create_test_spherical_solid();
        let scale_factor = 2.5;

        let scaled = spherical_solid.scale_uniform(scale_factor).unwrap();

        // 半径がスケール
        assert_relative_eq!(
            scaled.radius(),
            spherical_solid.radius() * scale_factor,
            epsilon = 1e-10
        );

        // 中心と軸は不変
        assert_eq!(scaled.center(), spherical_solid.center());
        assert_eq!(scaled.axis(), spherical_solid.axis());
        assert_eq!(scaled.ref_direction(), spherical_solid.ref_direction());

        // 無効な係数
        assert!(spherical_solid.scale_uniform(0.0).is_none());
        assert!(spherical_solid.scale_uniform(-1.0).is_none());
    }

    #[test]
    fn test_rotate_z() {
        let spherical_solid =
            SphericalSolid3D::new_standard(Point3D::new(1.0, 0.0, 0.0), 2.0).unwrap();

        let angle = Angle::from_degrees(90.0);
        let rotated = spherical_solid.rotate_z(&angle);

        // 90度回転後の確認
        assert_relative_eq!(rotated.center().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().z(), 0.0, epsilon = 1e-10);

        // 半径は不変
        assert_eq!(rotated.radius(), spherical_solid.radius());
    }

    #[test]
    fn test_direct_transform_methods() {
        let spherical_solid = create_test_spherical_solid();

        // 直接メソッド経由での操作
        let translation = Vector3D::new(1.0_f64, 1.0_f64, 1.0_f64);
        let translated = spherical_solid.translate(translation);
        assert_eq!(translated.center(), Point3D::new(2.0, 3.0, 4.0));

        let scaled = spherical_solid.scale_uniform(2.0).unwrap();
        assert_eq!(scaled.radius(), 4.0);

        let rotation_center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);
        let axis = Vector3D::unit_z();
        let rotated = spherical_solid
            .rotate_around_axis(&rotation_center, &axis, &angle)
            .unwrap();

        assert_relative_eq!(rotated.center().x(), -2.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_solid_property_preservation() {
        let spherical_solid = create_test_spherical_solid();
        let original_volume = spherical_solid.volume();

        // 平行移動では体積不変
        let translated = spherical_solid.translate(Vector3D::new(10.0, 20.0, 30.0));
        assert_relative_eq!(translated.volume(), original_volume, epsilon = 1e-10);

        // 回転では体積不変
        let angle = Angle::from_degrees(45.0);
        let rotated = spherical_solid.rotate_z(&angle);
        assert_relative_eq!(rotated.volume(), original_volume, epsilon = 1e-10);

        // スケールでは体積比例
        let scaled = spherical_solid.scale_uniform(2.0).unwrap();
        assert_relative_eq!(scaled.volume(), original_volume * 8.0, epsilon = 1e-10);
        // 2³
    }
}
