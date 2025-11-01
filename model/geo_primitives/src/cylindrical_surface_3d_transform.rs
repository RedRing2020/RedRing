//! CylindricalSurface3D変換機能の実装
//!
//! Foundation パターンによる統一Transform処理

use crate::{CylindricalSurface3D, Direction3D, Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

impl<T: Scalar> CylindricalSurface3D<T> {
    /// 平行移動
    pub fn translate(&self, translation: Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
        )
        .unwrap()
    }

    /// 非一様スケール（中心点指定）
    pub fn scale_non_uniform(
        &self,
        scale_center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Self {
        // 中心点のスケール変換
        let relative_center = Vector3D::new(
            self.center().x() - scale_center.x(),
            self.center().y() - scale_center.y(),
            self.center().z() - scale_center.z(),
        );
        let scaled_center = Point3D::new(
            scale_center.x() + relative_center.x() * scale_x,
            scale_center.y() + relative_center.y() * scale_y,
            scale_center.z() + relative_center.z() * scale_z,
        );

        // 円柱は等方的なスケールのみサポート（非一様スケールでは楕円柱になる）
        let scaled_radius = self.radius() * scale_x.abs();

        Self::new(
            scaled_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            scaled_radius,
        )
        .expect("CylindricalSurface3D creation should not fail with valid input")
    }

    /// 均一スケール（中心点指定）
    pub fn scale_uniform(&self, scale_center: Point3D<T>, factor: T) -> Self {
        self.scale_non_uniform(scale_center, factor, factor, factor)
    }

    /// Z軸周りの回転（2D回転、原点中心）
    pub fn rotate_z(&self, angle: Angle<T>) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 中心点のZ軸回転
        let new_center = Point3D::new(
            self.center().x() * cos_angle - self.center().y() * sin_angle,
            self.center().x() * sin_angle + self.center().y() * cos_angle,
            self.center().z(),
        );

        // 軸ベクトルのZ軸回転
        let axis_vec = self.axis().as_vector();
        let rotated_axis_vec = Vector3D::new(
            axis_vec.x() * cos_angle - axis_vec.y() * sin_angle,
            axis_vec.x() * sin_angle + axis_vec.y() * cos_angle,
            axis_vec.z(),
        );
        let new_axis = Direction3D::from_vector(rotated_axis_vec).unwrap();

        // 参照方向のZ軸回転
        let ref_vec = self.ref_direction().as_vector();
        let rotated_ref_vec = Vector3D::new(
            ref_vec.x() * cos_angle - ref_vec.y() * sin_angle,
            ref_vec.x() * sin_angle + ref_vec.y() * cos_angle,
            ref_vec.z(),
        );
        let new_ref = Direction3D::from_vector(rotated_ref_vec).unwrap();

        Self::new(
            new_center,
            new_axis.as_vector(),
            new_ref.as_vector(),
            self.radius(),
        )
        .unwrap()
    }

    /// 任意軸周りの回転
    pub fn rotate_around_axis(
        &self,
        axis: Vector3D<T>,
        angle: Angle<T>,
        rotation_center: Point3D<T>,
    ) -> Result<Self, &'static str> {
        // Rodrigues回転公式の実装
        fn rodrigues_rotation<T: Scalar>(
            v: Vector3D<T>,
            k: Vector3D<T>,
            cos_angle: T,
            sin_angle: T,
        ) -> Vector3D<T> {
            // v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
            let k_dot_v = k.dot(&v);
            let k_cross_v = k.cross(&v);
            let one_minus_cos = T::ONE - cos_angle;

            v * cos_angle + k_cross_v * sin_angle + k * k_dot_v * one_minus_cos
        }

        // 軸ベクトルを正規化
        let axis_length = axis.length();
        if axis_length.is_zero() {
            return Err("Cannot rotate around zero vector");
        }
        let axis_normalized = axis * (T::ONE / axis_length);

        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 中心点の回転
        let to_center = Vector3D::new(
            self.center().x() - rotation_center.x(),
            self.center().y() - rotation_center.y(),
            self.center().z() - rotation_center.z(),
        );
        let rotated_to_center =
            rodrigues_rotation(to_center, axis_normalized, cos_angle, sin_angle);
        let new_center = Point3D::new(
            rotation_center.x() + rotated_to_center.x(),
            rotation_center.y() + rotated_to_center.y(),
            rotation_center.z() + rotated_to_center.z(),
        );

        // 軸ベクトルの回転
        let rotated_axis = rodrigues_rotation(
            self.axis().as_vector(),
            axis_normalized,
            cos_angle,
            sin_angle,
        );
        let new_axis =
            Direction3D::from_vector(rotated_axis).ok_or("Invalid rotated axis direction")?;

        // 参照方向の回転
        let rotated_ref = rodrigues_rotation(
            self.ref_direction().as_vector(),
            axis_normalized,
            cos_angle,
            sin_angle,
        );
        let new_ref =
            Direction3D::from_vector(rotated_ref).ok_or("Invalid rotated ref direction")?;

        Self::new(
            new_center,
            new_axis.as_vector(),
            new_ref.as_vector(),
            self.radius(),
        )
        .ok_or("Failed to create rotated cylindrical surface")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_translate() {
        let surface = CylindricalSurface3D::new(
            Point3D::new(1.0, 2.0, 3.0),
            Vector3D::unit_z(), // Direction3DではなくVector3Dを使用
            Vector3D::unit_x(), // Direction3DではなくVector3Dを使用
            2.0,
        )
        .unwrap();

        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let translated = surface.translate(translation);

        assert_relative_eq!(translated.center().x(), 2.0);
        assert_relative_eq!(translated.center().y(), 3.0);
        assert_relative_eq!(translated.center().z(), 4.0);
        assert_relative_eq!(translated.radius(), 2.0);
    }

    #[test]
    fn test_scale_uniform() {
        let surface = CylindricalSurface3D::new(
            Point3D::new(2.0, 0.0, 0.0),
            Vector3D::unit_z(), // Direction3DではなくVector3Dを使用
            Vector3D::unit_x(), // Direction3DではなくVector3Dを使用
            1.0,
        )
        .unwrap();

        let scaled = surface.scale_uniform(Point3D::origin(), 2.0);

        assert_relative_eq!(scaled.center().x(), 4.0);
        assert_relative_eq!(scaled.center().y(), 0.0);
        assert_relative_eq!(scaled.center().z(), 0.0);
        assert_relative_eq!(scaled.radius(), 2.0);
    }

    #[test]
    fn test_rotate_z() {
        let surface = CylindricalSurface3D::new(
            Point3D::new(1.0, 0.0, 0.0),
            Vector3D::unit_z(), // Direction3DではなくVector3Dを使用
            Vector3D::unit_x(), // Direction3DではなくVector3Dを使用
            1.0,
        )
        .unwrap();

        let angle = Angle::from_radians(std::f64::consts::PI / 2.0);
        let rotated = surface.rotate_z(angle);

        assert_relative_eq!(rotated.center().x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.center().z(), 0.0);
    }
}
