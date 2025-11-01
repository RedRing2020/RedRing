//! ConicalSolid3D の安全な変換操作実装
//!
//! エラーハンドリングを含む ConicalSolid3D の変換操作を提供
//! 無効なパラメータや計算結果に対して適切なエラーを返す

use super::conical_solid_3d::ConicalSolid3D;
use crate::{Angle, Point3D, Vector3D};
use geo_foundation::{SafeTransform, Scalar, TransformError};

// ============================================================================
// Safe Transform Operations (安全な変換操作)
// ============================================================================

impl<T: Scalar> ConicalSolid3D<T> {
    /// 安全な平行移動
    ///
    /// 円錐を指定したベクトル分だけ移動
    /// 無効な変換ベクトル（NaN、無限大）をチェック
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 移動後の新しい円錐ソリッド、または変換エラー
    pub fn translate_safe(&self, translation: &Vector3D<T>) -> Result<Self, TransformError> {
        // 変換ベクトルの検証
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::ZeroVector(
                "Translation vector contains non-finite values".to_string(),
            ));
        }

        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );

        // 結果の検証
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "Translation resulted in non-finite center".to_string(),
            ));
        }

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.height(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create cone after translation".to_string())
        })
    }

    /// 安全なZ軸周りの回転
    ///
    /// 円錐を指定角度だけZ軸周りに回転
    /// 無効な角度（NaN、無限大）をチェック
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の新しい円錐ソリッド、または変換エラー
    pub fn rotate_z_safe(&self, angle: T) -> Result<Self, TransformError> {
        // 角度の検証
        if !angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "Rotation angle is not finite".to_string(),
            ));
        }

        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        // 三角関数の結果検証
        if !cos_theta.is_finite() || !sin_theta.is_finite() {
            return Err(TransformError::InvalidRotation(
                "Trigonometric functions resulted in non-finite values".to_string(),
            ));
        }

        // 回転行列による軸の変換
        let new_axis = Vector3D::new(
            self.axis().x() * cos_theta - self.axis().y() * sin_theta,
            self.axis().x() * sin_theta + self.axis().y() * cos_theta,
            self.axis().z(),
        );

        // 回転行列による参照方向の変換
        let new_ref_direction = Vector3D::new(
            self.ref_direction().x() * cos_theta - self.ref_direction().y() * sin_theta,
            self.ref_direction().x() * sin_theta + self.ref_direction().y() * cos_theta,
            self.ref_direction().z(),
        );

        // 結果の検証
        if !new_axis.x().is_finite() || !new_axis.y().is_finite() || !new_axis.z().is_finite() {
            return Err(TransformError::InvalidRotation(
                "Rotation resulted in non-finite axis vector".to_string(),
            ));
        }

        if !new_ref_direction.x().is_finite()
            || !new_ref_direction.y().is_finite()
            || !new_ref_direction.z().is_finite()
        {
            return Err(TransformError::InvalidRotation(
                "Rotation resulted in non-finite reference direction".to_string(),
            ));
        }

        Self::new(
            self.center(),
            new_axis,
            new_ref_direction,
            self.radius(),
            self.height(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create cone after rotation".to_string())
        })
    }

    /// 安全な任意軸周りの回転
    ///
    /// 指定された軸周りに円錐を回転
    /// Rodriguesの回転公式を使用
    ///
    /// # Arguments
    /// * `axis` - 回転軸ベクトル（単位ベクトルでなくても可）
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の新しい円錐ソリッド、または変換エラー
    pub fn rotate_axis_safe(&self, axis: &Vector3D<T>, angle: T) -> Result<Self, TransformError> {
        // 角度の検証
        if !angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "Rotation angle is not finite".to_string(),
            ));
        }

        // 軸ベクトルの検証
        if !axis.x().is_finite() || !axis.y().is_finite() || !axis.z().is_finite() {
            return Err(TransformError::ZeroVector(
                "Rotation axis contains non-finite values".to_string(),
            ));
        }

        let axis_length = axis.length();
        if axis_length <= T::EPSILON {
            return Err(TransformError::ZeroVector(
                "Rotation axis is too short or zero".to_string(),
            ));
        }

        // 軸を正規化
        let normalized_axis = Vector3D::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );

        // Rodriguesの回転公式を適用するヘルパー関数
        let rotate_vector = |v: &Vector3D<T>| -> Result<Vector3D<T>, TransformError> {
            let cos_theta = angle.cos();
            let sin_theta = angle.sin();

            // v × k (外積)
            let cross = Vector3D::new(
                v.y() * normalized_axis.z() - v.z() * normalized_axis.y(),
                v.z() * normalized_axis.x() - v.x() * normalized_axis.z(),
                v.x() * normalized_axis.y() - v.y() * normalized_axis.x(),
            );

            // k · v (内積)
            let dot = v.x() * normalized_axis.x()
                + v.y() * normalized_axis.y()
                + v.z() * normalized_axis.z();

            // Rodrigues公式: v' = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
            let result = Vector3D::new(
                v.x() * cos_theta
                    + cross.x() * sin_theta
                    + normalized_axis.x() * dot * (T::ONE - cos_theta),
                v.y() * cos_theta
                    + cross.y() * sin_theta
                    + normalized_axis.y() * dot * (T::ONE - cos_theta),
                v.z() * cos_theta
                    + cross.z() * sin_theta
                    + normalized_axis.z() * dot * (T::ONE - cos_theta),
            );

            // 結果の検証
            if !result.x().is_finite() || !result.y().is_finite() || !result.z().is_finite() {
                return Err(TransformError::InvalidRotation(
                    "Rodrigues rotation resulted in non-finite vector".to_string(),
                ));
            }

            Ok(result)
        };

        // 軸と参照方向を回転
        let rotated_axis = rotate_vector(&self.axis().as_vector())?;
        let rotated_ref_direction = rotate_vector(&self.ref_direction().as_vector())?;

        Self::new(
            self.center(),
            rotated_axis,
            rotated_ref_direction,
            self.radius(),
            self.height(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create cone after axis rotation".to_string())
        })
    }

    /// 安全な等方スケール
    ///
    /// 円錐の半径と高さを指定倍率でスケール
    /// 無効な倍率（負数、ゼロ、NaN、無限大）をチェック
    ///
    /// # Arguments
    /// * `factor` - スケール倍率（正の値）
    ///
    /// # Returns
    /// スケール後の新しい円錐ソリッド、または変換エラー
    pub fn scale_uniform_safe(&self, factor: T) -> Result<Self, TransformError> {
        // 倍率の検証
        if !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor is not finite".to_string(),
            ));
        }

        if factor <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor must be positive".to_string(),
            ));
        }

        let new_radius = self.radius() * factor;
        let new_height = self.height() * factor;

        // 結果の検証
        if !new_radius.is_finite() || !new_height.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scaling resulted in non-finite dimensions".to_string(),
            ));
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            new_height,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create cone after scaling".to_string())
        })
    }

    /// 安全なミラーリング（反射）
    ///
    /// 指定された法線ベクトルを持つ平面で円錐を反射
    ///
    /// # Arguments
    /// * `normal` - 反射平面の法線ベクトル
    ///
    /// # Returns
    /// 反射後の新しい円錐ソリッド、または変換エラー
    pub fn reflect_safe(&self, normal: &Vector3D<T>) -> Result<Self, TransformError> {
        // 法線ベクトルの検証
        if !normal.x().is_finite() || !normal.y().is_finite() || !normal.z().is_finite() {
            return Err(TransformError::ZeroVector(
                "Reflection normal contains non-finite values".to_string(),
            ));
        }

        let normal_length = normal.length();
        if normal_length <= T::EPSILON {
            return Err(TransformError::ZeroVector(
                "Reflection normal is too short or zero".to_string(),
            ));
        }

        // 法線を正規化
        let normalized_normal = Vector3D::new(
            normal.x() / normal_length,
            normal.y() / normal_length,
            normal.z() / normal_length,
        );

        // ベクトルの反射: v' = v - 2(v·n)n
        let reflect_vector = |v: &Vector3D<T>| -> Result<Vector3D<T>, TransformError> {
            let dot = v.x() * normalized_normal.x()
                + v.y() * normalized_normal.y()
                + v.z() * normalized_normal.z();
            let two_dot = dot + dot; // 2 * dot

            let result = Vector3D::new(
                v.x() - two_dot * normalized_normal.x(),
                v.y() - two_dot * normalized_normal.y(),
                v.z() - two_dot * normalized_normal.z(),
            );

            // 結果の検証
            if !result.x().is_finite() || !result.y().is_finite() || !result.z().is_finite() {
                return Err(TransformError::InvalidRotation(
                    "Reflection resulted in non-finite vector".to_string(),
                ));
            }

            Ok(result)
        };

        // 点の反射
        let center_vector = Vector3D::new(self.center().x(), self.center().y(), self.center().z());
        let reflected_center_vector = reflect_vector(&center_vector)?;
        let reflected_center = Point3D::new(
            reflected_center_vector.x(),
            reflected_center_vector.y(),
            reflected_center_vector.z(),
        );

        // 軸と参照方向を反射
        let reflected_axis = reflect_vector(&self.axis().as_vector())?;
        let reflected_ref_direction = reflect_vector(&self.ref_direction().as_vector())?;

        Self::new(
            reflected_center,
            reflected_axis,
            reflected_ref_direction,
            self.radius(),
            self.height(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("Failed to create cone after reflection".to_string())
        })
    }
}

// ============================================================================
// SafeTransform Trait Implementation (Foundation パターン)
// ============================================================================

impl<T: Scalar> SafeTransform<T> for ConicalSolid3D<T> {
    /// 安全な平行移動（Foundation パターン）
    fn safe_translate(&self, offset: T) -> Result<Self, TransformError> {
        // オフセットをZ軸方向の移動として解釈
        let translation = Vector3D::new(T::ZERO, T::ZERO, offset);
        self.translate_safe(&translation)
    }

    /// 安全な指定中心でのスケール
    fn safe_scale(&self, _center: T, factor: T) -> Result<Self, TransformError> {
        self.scale_uniform_safe(factor)
    }

    /// 安全な回転（Z軸周り回転として実装）
    fn safe_rotate(&self, _center: T, _axis: T, angle: Angle<T>) -> Result<Self, TransformError> {
        self.rotate_z_safe(angle.to_radians())
    }
}

// ============================================================================
// Tests (テスト)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};

    fn create_test_cone() -> ConicalSolid3D<f64> {
        ConicalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            2.0,
        )
        .unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let cone = create_test_cone();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let result = cone.translate_safe(&translation);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.center(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(transformed.radius(), 1.0);
        assert_eq!(transformed.height(), 2.0);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let cone = create_test_cone();
        let invalid_translation = Vector3D::new(f64::NAN, 0.0, 0.0);

        let result = cone.translate_safe(&invalid_translation);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_rotate_z_success() {
        let cone = create_test_cone();
        let angle = std::f64::consts::PI / 2.0; // 90度

        let result = cone.rotate_z_safe(angle);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // 90度回転後、参照方向 (1,0,0) は (0,1,0) になる
        let expected_ref = Vector3D::new(0.0, 1.0, 0.0);
        let actual_ref = transformed.ref_direction().as_vector();

        assert!((actual_ref.x() - expected_ref.x()).abs() < 1e-10);
        assert!((actual_ref.y() - expected_ref.y()).abs() < 1e-10);
        assert!((actual_ref.z() - expected_ref.z()).abs() < 1e-10);
    }

    #[test]
    fn test_safe_rotate_z_invalid_angle() {
        let cone = create_test_cone();
        let invalid_angle = f64::NAN;

        let result = cone.rotate_z_safe(invalid_angle);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));
    }

    #[test]
    fn test_safe_rotate_axis_success() {
        let cone = create_test_cone();
        let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸
        let angle = std::f64::consts::PI / 2.0; // 90度

        let result = cone.rotate_axis_safe(&axis, angle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_rotate_axis_zero_axis_error() {
        let cone = create_test_cone();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = std::f64::consts::PI / 2.0;

        let result = cone.rotate_axis_safe(&zero_axis, angle);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_scale_uniform_success() {
        let cone = create_test_cone();
        let factor = 2.0;

        let result = cone.scale_uniform_safe(factor);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.radius(), 2.0);
        assert_eq!(transformed.height(), 4.0);
        assert_eq!(transformed.center(), cone.center()); // 中心は変化しない
    }

    #[test]
    fn test_safe_scale_uniform_zero_factor_error() {
        let cone = create_test_cone();
        let zero_factor = 0.0;

        let result = cone.scale_uniform_safe(zero_factor);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_safe_scale_uniform_negative_factor_error() {
        let cone = create_test_cone();
        let negative_factor = -1.0;

        let result = cone.scale_uniform_safe(negative_factor);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidScaleFactor(_)
        ));
    }

    #[test]
    fn test_safe_reflect_success() {
        let cone = create_test_cone();
        let normal = Vector3D::new(1.0, 0.0, 0.0); // X軸法線

        let result = cone.reflect_safe(&normal);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        // X軸での反射により、参照方向 (1,0,0) は (-1,0,0) になる
        let expected_ref = Vector3D::new(-1.0, 0.0, 0.0);
        let actual_ref = transformed.ref_direction().as_vector();

        assert!((actual_ref.x() - expected_ref.x()).abs() < 1e-10);
        assert!((actual_ref.y() - expected_ref.y()).abs() < 1e-10);
        assert!((actual_ref.z() - expected_ref.z()).abs() < 1e-10);
    }

    #[test]
    fn test_safe_reflect_zero_normal_error() {
        let cone = create_test_cone();
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);

        let result = cone.reflect_safe(&zero_normal);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::ZeroVector(_)));
    }

    #[test]
    fn test_safe_transform_trait() {
        let cone = create_test_cone();
        let offset = 1.0; // Z軸方向の移動

        let result = cone.safe_translate(offset);
        assert!(result.is_ok());

        let transformed = result.unwrap();
        assert_eq!(transformed.center(), Point3D::new(0.0, 0.0, 1.0));
    }
}
