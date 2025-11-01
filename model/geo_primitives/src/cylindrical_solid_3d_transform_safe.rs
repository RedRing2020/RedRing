//! CylindricalSolid3D Safe Transform Operations
//!
//! STEP準拠円柱ソリッドの安全な変換操作実装（エラーハンドリング対応版）
//!
//! **作成日: 2025年11月1日**
//! **最終更新: 2025年11月1日**
//!
//! ## 実装内容
//! - 平行移動：center の安全な移動
//! - 回転：軸と参照方向の安全な回転
//! - スケール：半径・高さの安全なスケーリング
//! - 包括的エラーハンドリング：Result型による安全な変換
//! - SafeTransform トレイト実装（エラー安全な変換パターン）
//!
//! ## STEP準拠円柱ソリッド変換の特性
//! - 軸と参照方向の直交性保持：変換後も axis ⊥ ref_direction を維持
//! - 正規化保持：軸と参照方向が単位ベクトルのまま
//! - 右手系保持：Y軸 = Z軸 × X軸 関係を維持
//! - 幾何学的整合性：半径・高さの正の値保持
//! - ソリッド特性保持：体積比例、内部判定整合性
//! - エラー安全性：無効な変換パラメータでの失敗を明示的に処理

use crate::{CylindricalSolid3D, Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Error Types
// ============================================================================

/// 円柱ソリッド変換エラー
#[derive(Debug, Clone, PartialEq)]
pub enum CylindricalSolid3DTransformError {
    /// 無効なスケール係数（0以下の値）
    InvalidScaleFactor { factor: String, reason: String },
    /// 無効な回転軸（ゼロベクトル）
    InvalidRotationAxis { axis: String, reason: String },
    /// 円柱構築失敗（不正な幾何パラメータ）
    CylinderConstructionFailed { operation: String, reason: String },
    /// 数値計算エラー
    NumericalError { operation: String, reason: String },
}

impl std::fmt::Display for CylindricalSolid3DTransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidScaleFactor { factor, reason } => {
                write!(f, "Invalid scale factor '{}': {}", factor, reason)
            }
            Self::InvalidRotationAxis { axis, reason } => {
                write!(f, "Invalid rotation axis '{}': {}", axis, reason)
            }
            Self::CylinderConstructionFailed { operation, reason } => {
                write!(
                    f,
                    "Cylinder construction failed during '{}': {}",
                    operation, reason
                )
            }
            Self::NumericalError { operation, reason } => {
                write!(f, "Numerical error during '{}': {}", operation, reason)
            }
        }
    }
}

impl std::error::Error for CylindricalSolid3DTransformError {}

/// 変換結果型の便利なエイリアス
pub type TransformResult<T> = Result<CylindricalSolid3D<T>, CylindricalSolid3DTransformError>;

// ============================================================================
// Safe Transform Operations
// ============================================================================

impl<T: Scalar> CylindricalSolid3D<T> {
    /// 安全な平行移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 平行移動後の円柱ソリッド、または変換エラー
    ///
    /// # Note
    /// 軸方向、参照方向、半径、高さは変更されず、center のみ移動
    /// 数値的に有効な平行移動のみ許可
    pub fn translate_safe(&self, translation: &Vector3D<T>) -> TransformResult<T> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "translate".to_string(),
                reason: "Translation vector contains non-finite values".to_string(),
            });
        }

        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );

        // 新しい中心点の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "translate".to_string(),
                reason: "Translated center contains non-finite values".to_string(),
            });
        }

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.height(),
        )
        .ok_or_else(
            || CylindricalSolid3DTransformError::CylinderConstructionFailed {
                operation: "translate".to_string(),
                reason: "Failed to construct cylinder with translated center".to_string(),
            },
        )
    }

    /// 安全な均等スケール変換
    ///
    /// # Arguments
    /// * `scale_factor` - スケール係数（正の値）
    ///
    /// # Returns
    /// スケール変換後の円柱ソリッド、または変換エラー
    ///
    /// # Note
    /// 中心、軸方向、参照方向はそのまま、半径と高さを scale_factor 倍
    pub fn scale_uniform_safe(&self, scale_factor: T) -> TransformResult<T> {
        // スケール係数の有効性チェック
        if !scale_factor.is_finite() {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("{:?}", scale_factor),
                reason: "Scale factor is not finite".to_string(),
            });
        }

        if scale_factor <= T::ZERO {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("{:?}", scale_factor),
                reason: "Scale factor must be positive".to_string(),
            });
        }

        let new_radius = self.radius() * scale_factor;
        let new_height = self.height() * scale_factor;

        // 結果の有効性チェック
        if !new_radius.is_finite() || !new_height.is_finite() {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "scale_uniform".to_string(),
                reason: "Scaled dimensions contain non-finite values".to_string(),
            });
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            new_height,
        )
        .ok_or_else(
            || CylindricalSolid3DTransformError::CylinderConstructionFailed {
                operation: "scale_uniform".to_string(),
                reason: "Failed to construct cylinder with scaled dimensions".to_string(),
            },
        )
    }

    /// 安全な非均等スケール変換
    ///
    /// # Arguments
    /// * `radius_scale` - 半径のスケール係数（正の値）
    /// * `height_scale` - 高さのスケール係数（正の値）
    ///
    /// # Returns
    /// 非均等スケール変換後の円柱ソリッド、または変換エラー
    ///
    /// # Note
    /// 中心、軸方向、参照方向はそのまま、半径と高さを個別にスケール
    pub fn scale_non_uniform_safe(&self, radius_scale: T, height_scale: T) -> TransformResult<T> {
        // スケール係数の有効性チェック
        if !radius_scale.is_finite() {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("radius_scale: {:?}", radius_scale),
                reason: "Radius scale factor is not finite".to_string(),
            });
        }

        if !height_scale.is_finite() {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("height_scale: {:?}", height_scale),
                reason: "Height scale factor is not finite".to_string(),
            });
        }

        if radius_scale <= T::ZERO {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("radius_scale: {:?}", radius_scale),
                reason: "Radius scale factor must be positive".to_string(),
            });
        }

        if height_scale <= T::ZERO {
            return Err(CylindricalSolid3DTransformError::InvalidScaleFactor {
                factor: format!("height_scale: {:?}", height_scale),
                reason: "Height scale factor must be positive".to_string(),
            });
        }

        let new_radius = self.radius() * radius_scale;
        let new_height = self.height() * height_scale;

        // 結果の有効性チェック
        if !new_radius.is_finite() || !new_height.is_finite() {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "scale_non_uniform".to_string(),
                reason: "Scaled dimensions contain non-finite values".to_string(),
            });
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            new_height,
        )
        .ok_or_else(
            || CylindricalSolid3DTransformError::CylinderConstructionFailed {
                operation: "scale_non_uniform".to_string(),
                reason: "Failed to construct cylinder with non-uniformly scaled dimensions"
                    .to_string(),
            },
        )
    }

    /// 安全な任意軸周りの回転
    ///
    /// # Arguments
    /// * `rotation_center` - 回転中心点
    /// * `rotation_axis` - 回転軸（正規化される）
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// 回転後の円柱ソリッド、または変換エラー
    ///
    /// # Note
    /// center, axis, ref_direction を回転軸周りに回転
    /// 半径、高さは不変
    pub fn rotate_around_axis_safe(
        &self,
        rotation_center: &Point3D<T>,
        rotation_axis: &Vector3D<T>,
        angle: &Angle<T>,
    ) -> TransformResult<T> {
        // 回転中心の有効性チェック
        if !rotation_center.x().is_finite()
            || !rotation_center.y().is_finite()
            || !rotation_center.z().is_finite()
        {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "rotate_around_axis".to_string(),
                reason: "Rotation center contains non-finite values".to_string(),
            });
        }

        // 回転軸の有効性チェック
        if !rotation_axis.x().is_finite()
            || !rotation_axis.y().is_finite()
            || !rotation_axis.z().is_finite()
        {
            return Err(CylindricalSolid3DTransformError::InvalidRotationAxis {
                axis: format!("{:?}", rotation_axis),
                reason: "Rotation axis contains non-finite values".to_string(),
            });
        }

        let axis_norm = rotation_axis.magnitude();
        if axis_norm.is_zero() {
            return Err(CylindricalSolid3DTransformError::InvalidRotationAxis {
                axis: format!("{:?}", rotation_axis),
                reason: "Rotation axis is zero vector".to_string(),
            });
        }

        // 角度の有効性チェック
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        if !cos_theta.is_finite() || !sin_theta.is_finite() {
            return Err(CylindricalSolid3DTransformError::NumericalError {
                operation: "rotate_around_axis".to_string(),
                reason: "Angle trigonometric values are not finite".to_string(),
            });
        }

        let normalized_axis = *rotation_axis / axis_norm;

        // Rodrigues の回転公式のヘルパー関数
        let rotate_vector =
            |v: &Vector3D<T>| -> Result<Vector3D<T>, CylindricalSolid3DTransformError> {
                let k = normalized_axis;
                let v_parallel = k * v.dot(&k);
                let v_perpendicular = *v - v_parallel;
                let w = k.cross(v);

                let rotated = v_parallel + v_perpendicular * cos_theta + w * sin_theta;

                // 結果の有効性チェック
                if !rotated.x().is_finite() || !rotated.y().is_finite() || !rotated.z().is_finite()
                {
                    return Err(CylindricalSolid3DTransformError::NumericalError {
                        operation: "rotate_vector".to_string(),
                        reason: "Rotated vector contains non-finite values".to_string(),
                    });
                }

                Ok(rotated)
            };

        // 中心点の回転
        let center_vec = Vector3D::new(
            self.center().x() - rotation_center.x(),
            self.center().y() - rotation_center.y(),
            self.center().z() - rotation_center.z(),
        );
        let rotated_center_vec = rotate_vector(&center_vec)?;
        let new_center = Point3D::new(
            rotation_center.x() + rotated_center_vec.x(),
            rotation_center.y() + rotated_center_vec.y(),
            rotation_center.z() + rotated_center_vec.z(),
        );

        // 軸の回転
        let rotated_axis = rotate_vector(&self.axis().as_vector())?;

        // 参照方向の回転
        let rotated_ref_direction = rotate_vector(&self.ref_direction().as_vector())?;

        Self::new(
            new_center,
            rotated_axis,
            rotated_ref_direction,
            self.radius(),
            self.height(),
        )
        .ok_or_else(
            || CylindricalSolid3DTransformError::CylinderConstructionFailed {
                operation: "rotate_around_axis".to_string(),
                reason: "Failed to construct cylinder with rotated geometry".to_string(),
            },
        )
    }

    /// 安全なX軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// X軸周りに回転した円柱ソリッド、または変換エラー
    pub fn rotate_x_safe(&self, angle: &Angle<T>) -> TransformResult<T> {
        self.rotate_around_axis_safe(
            &Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            &Vector3D::new(T::ONE, T::ZERO, T::ZERO),
            angle,
        )
    }

    /// 安全なY軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// Y軸周りに回転した円柱ソリッド、または変換エラー
    pub fn rotate_y_safe(&self, angle: &Angle<T>) -> TransformResult<T> {
        self.rotate_around_axis_safe(
            &Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            &Vector3D::new(T::ZERO, T::ONE, T::ZERO),
            angle,
        )
    }

    /// 安全なZ軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// Z軸周りに回転した円柱ソリッド、または変換エラー
    pub fn rotate_z_safe(&self, angle: &Angle<T>) -> TransformResult<T> {
        self.rotate_around_axis_safe(
            &Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            &Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            angle,
        )
    }
}

// ============================================================================
// SafeTransform Trait (Foundation パターン準拠)
// ============================================================================

/// 安全な変換操作を提供するトレイト
pub trait SafeTransform<T: Scalar> {
    type Transformed;
    type Error;

    /// 安全な平行移動
    fn translate_safe(&self, translation: &Vector3D<T>) -> Result<Self::Transformed, Self::Error>;

    /// 安全な回転（指定中心周り）
    fn rotate_safe(
        &self,
        center: &Point3D<T>,
        angle: &Angle<T>,
    ) -> Result<Self::Transformed, Self::Error>;

    /// 安全なスケール（指定中心周り）
    fn scale_safe(&self, center: &Point3D<T>, factor: T) -> Result<Self::Transformed, Self::Error>;
}

impl<T: Scalar> SafeTransform<T> for CylindricalSolid3D<T> {
    type Transformed = Self;
    type Error = CylindricalSolid3DTransformError;

    /// 安全な平行移動（SafeTransform トレイト実装）
    fn translate_safe(&self, translation: &Vector3D<T>) -> Result<Self::Transformed, Self::Error> {
        self.translate_safe(translation)
    }

    /// 安全な回転（指定中心での Z軸周り回転として実装）
    fn rotate_safe(
        &self,
        center: &Point3D<T>,
        angle: &Angle<T>,
    ) -> Result<Self::Transformed, Self::Error> {
        self.rotate_around_axis_safe(center, &Vector3D::new(T::ZERO, T::ZERO, T::ONE), angle)
    }

    /// 安全なスケール（指定中心での均等スケールとして実装）
    ///
    /// # Note
    /// 中心点は無視され、円柱の center, radius, height がスケールされる
    fn scale_safe(
        &self,
        _center: &Point3D<T>,
        factor: T,
    ) -> Result<Self::Transformed, Self::Error> {
        self.scale_uniform_safe(factor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_cylinder() -> CylindricalSolid3D<f64> {
        CylindricalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
            2.0,
        )
        .unwrap()
    }

    #[test]
    fn test_translate_safe_success() {
        let cylinder = create_test_cylinder();
        let translation = Vector3D::new(1.0, 2.0, 3.0);

        let result = cylinder.translate_safe(&translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        assert_eq!(translated.center(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(translated.radius(), 1.0);
        assert_eq!(translated.height(), 2.0);
    }

    #[test]
    fn test_translate_safe_invalid_vector() {
        let cylinder = create_test_cylinder();
        let translation = Vector3D::new(f64::NAN, 2.0, 3.0);

        let result = cylinder.translate_safe(&translation);
        assert!(result.is_err());

        if let Err(CylindricalSolid3DTransformError::NumericalError { operation, .. }) = result {
            assert_eq!(operation, "translate");
        } else {
            panic!("Expected NumericalError");
        }
    }

    #[test]
    fn test_scale_uniform_safe_success() {
        let cylinder = create_test_cylinder();

        let result = cylinder.scale_uniform_safe(2.0);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        assert_eq!(scaled.radius(), 2.0);
        assert_eq!(scaled.height(), 4.0);
    }

    #[test]
    fn test_scale_uniform_safe_invalid_factor() {
        let cylinder = create_test_cylinder();

        let result = cylinder.scale_uniform_safe(-1.0);
        assert!(result.is_err());

        if let Err(CylindricalSolid3DTransformError::InvalidScaleFactor { .. }) = result {
            // Expected
        } else {
            panic!("Expected InvalidScaleFactor");
        }
    }

    #[test]
    fn test_scale_uniform_safe_zero_factor() {
        let cylinder = create_test_cylinder();

        let result = cylinder.scale_uniform_safe(0.0);
        assert!(result.is_err());

        if let Err(CylindricalSolid3DTransformError::InvalidScaleFactor { .. }) = result {
            // Expected
        } else {
            panic!("Expected InvalidScaleFactor");
        }
    }

    #[test]
    fn test_rotate_around_axis_safe_success() {
        let cylinder = create_test_cylinder();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 1.0);
        let angle = Angle::from_degrees(90.0);

        let result = cylinder.rotate_around_axis_safe(&center, &axis, &angle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rotate_around_axis_safe_zero_axis() {
        let cylinder = create_test_cylinder();
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);

        let result = cylinder.rotate_around_axis_safe(&center, &axis, &angle);
        assert!(result.is_err());

        if let Err(CylindricalSolid3DTransformError::InvalidRotationAxis { .. }) = result {
            // Expected
        } else {
            panic!("Expected InvalidRotationAxis");
        }
    }

    #[test]
    fn test_safe_transform_trait() {
        let cylinder = create_test_cylinder();
        let translation = Vector3D::new(1.0, 0.0, 0.0);
        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(45.0);

        // SafeTransform トレイトメソッドのテスト
        let translated = cylinder.translate_safe(&translation);
        assert!(translated.is_ok());

        let rotated = cylinder.rotate_safe(&center, &angle);
        assert!(rotated.is_ok());

        let scaled = cylinder.scale_safe(&center, 2.0);
        assert!(scaled.is_ok());
    }
}
