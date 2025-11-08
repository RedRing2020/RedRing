// torus_surface_3d_transform.rs
// TorusSurface3D の BasicTransform トレイト実装
//
// 基本的な幾何変換操作（平行移動、回転、スケール）をトーラス面に適用します。

use crate::{Point3D, TorusSurface3D, Vector3D};
use analysis::abstract_types::Angle;
use geo_foundation::{BasicTransform, Scalar};

impl<T: Scalar> BasicTransform<T> for TorusSurface3D<T> {
    type Transformed = Self;
    type Vector2D = Vector3D<T>; // 3Dでは2Dベクトル型として3Dベクトル使用
    type Point2D = Point3D<T>; // 3Dでは2D点型として3D点使用
    type Angle = Angle<T>;

    /// 平行移動を適用
    ///
    /// トーラス面の原点を指定ベクトルだけ移動します。
    /// 軸の向きや半径は変更されません。
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let current_origin = self.origin();
        let new_origin = Point3D::new(
            current_origin.x() + translation.x(),
            current_origin.y() + translation.y(),
            current_origin.z() + translation.z(),
        );

        // 新しい原点で再構築
        TorusSurface3D::new(
            new_origin,
            self.z_axis(),
            self.x_axis(),
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("Translation should preserve validity")
    }

    /// 指定点を中心とした回転を適用（Z軸周りの回転として実装）
    ///
    /// 2D回転としてZ軸周りの回転を適用します。
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // center 周りの回転
        let current_origin = self.origin();
        let relative_origin = Point3D::new(
            current_origin.x() - center.x(),
            current_origin.y() - center.y(),
            current_origin.z() - center.z(),
        );

        let rotated_relative = Point3D::new(
            cos_a * relative_origin.x() - sin_a * relative_origin.y(),
            sin_a * relative_origin.x() + cos_a * relative_origin.y(),
            relative_origin.z(),
        );

        let new_origin = Point3D::new(
            center.x() + rotated_relative.x(),
            center.y() + rotated_relative.y(),
            center.z() + rotated_relative.z(),
        );

        // 軸の回転（Z軸周りなのでX軸とY軸のみ回転）
        let current_x_axis = self.x_axis();
        let rotated_x_vec = Vector3D::new(
            cos_a * current_x_axis.x() - sin_a * current_x_axis.y(),
            sin_a * current_x_axis.x() + cos_a * current_x_axis.y(),
            current_x_axis.z(),
        );
        let new_x_axis = crate::Direction3D::from_vector(rotated_x_vec)
            .expect("Rotation should preserve unit vectors");

        // Z軸は変化しない
        TorusSurface3D::new(
            new_origin,
            self.z_axis(),
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("Rotation should preserve validity")
    }

    /// 指定点を中心としたスケールを適用
    ///
    /// 均等スケールをトーラス面に適用します。
    /// 半径がスケールされ、原点が中心点からスケールされます。
    fn scale(&self, scale_center: Self::Point2D, scale_factor: T) -> Self::Transformed {
        // スケールファクタが正の値でない場合は元のトーラスを返す
        if scale_factor <= T::ZERO {
            return self.clone();
        }

        // 原点のスケール
        let current_origin = self.origin();
        let relative_origin = Vector3D::new(
            current_origin.x() - scale_center.x(),
            current_origin.y() - scale_center.y(),
            current_origin.z() - scale_center.z(),
        );

        let scaled_relative_origin = Vector3D::new(
            relative_origin.x() * scale_factor,
            relative_origin.y() * scale_factor,
            relative_origin.z() * scale_factor,
        );

        let new_origin = Point3D::new(
            scale_center.x() + scaled_relative_origin.x(),
            scale_center.y() + scaled_relative_origin.y(),
            scale_center.z() + scaled_relative_origin.z(),
        );

        // 半径のスケール
        let new_major_radius = self.major_radius() * scale_factor;
        let new_minor_radius = self.minor_radius() * scale_factor;

        // 軸の向きは変更しない（スケールは均等なので）
        TorusSurface3D::new(
            new_origin,
            self.z_axis(),
            self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .expect("Scale should preserve validity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::abstract_types::Angle;
    use std::f64::consts::PI;

    #[test]
    fn test_translate() {
        let torus = TorusSurface3D::standard(3.0, 1.0).unwrap();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let translated_torus = torus.translate(translation);

        let new_origin = translated_torus.origin();
        assert!((new_origin.x() - 1.0).abs() < 1e-10);
        assert!((new_origin.y() - 2.0).abs() < 1e-10);
        assert!((new_origin.z() - 3.0).abs() < 1e-10);

        // 半径と軸は変更されない
        assert!((translated_torus.major_radius() - 3.0).abs() < 1e-10);
        assert!((translated_torus.minor_radius() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotate_around_origin() {
        let torus = TorusSurface3D::standard(3.0, 1.0).unwrap();
        let rotation_center = Point3D::origin();
        let angle = Angle::from_radians(PI / 2.0); // 90度回転

        let rotated_torus = torus.rotate(rotation_center, angle);

        // X軸がY軸方向に回転される（Z軸周りの回転）
        let new_x_axis = rotated_torus.x_axis();
        assert!((new_x_axis.y() - 1.0).abs() < 1e-10);
        assert!(new_x_axis.x().abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let torus = TorusSurface3D::standard(3.0, 1.0).unwrap();
        let scale_center = Point3D::origin();
        let scale_factor = 2.0;

        let scaled_torus = torus.scale(scale_center, scale_factor);

        // 半径が2倍になる
        assert!((scaled_torus.major_radius() - 6.0).abs() < 1e-10);
        assert!((scaled_torus.minor_radius() - 2.0).abs() < 1e-10);

        // 原点中心のスケールなので原点は変化しない
        let origin = scaled_torus.origin();
        assert!(origin.x().abs() < 1e-10);
        assert!(origin.y().abs() < 1e-10);
        assert!(origin.z().abs() < 1e-10);
    }

    #[test]
    fn test_scale_with_offset_center() {
        let torus = TorusSurface3D::new(
            Point3D::new(2.0, 0.0, 0.0),
            crate::Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            crate::Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(),
            3.0,
            1.0,
        )
        .unwrap();

        let scale_center = Point3D::origin();
        let scale_factor = 2.0;

        let scaled_torus = torus.scale(scale_center, scale_factor);

        // 原点もスケールされる
        let new_origin = scaled_torus.origin();
        assert!((new_origin.x() - 4.0).abs() < 1e-10);
        assert!(new_origin.y().abs() < 1e-10);
        assert!(new_origin.z().abs() < 1e-10);

        // 半径もスケールされる
        assert!((scaled_torus.major_radius() - 6.0).abs() < 1e-10);
        assert!((scaled_torus.minor_radius() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_combined_transforms() {
        let torus = TorusSurface3D::standard(2.0, 0.5).unwrap();

        // 1. 平行移動
        let translated = torus.translate(Vector3D::new(1.0, 1.0, 0.0));

        // 2. スケール
        let scaled = translated.scale(Point3D::origin(), 2.0);

        // 3. 回転
        let angle = Angle::from_radians(PI / 4.0);
        let final_torus = scaled.rotate(Point3D::origin(), angle);

        // 変換が正常に適用されているかチェック
        assert!(final_torus.is_valid());
        assert!(final_torus.major_radius() > 0.0);
        assert!(final_torus.minor_radius() > 0.0);
    }

    #[test]
    fn test_scale_with_zero_factor() {
        let torus = TorusSurface3D::standard(3.0, 1.0).unwrap();
        let original_major = torus.major_radius();
        let original_minor = torus.minor_radius();

        // ゼロスケールファクタ（無効）
        let result_torus = torus.scale(Point3D::origin(), 0.0);

        // 元のトーラスが返される
        assert!((result_torus.major_radius() - original_major).abs() < 1e-10);
        assert!((result_torus.minor_radius() - original_minor).abs() < 1e-10);
    }
}
