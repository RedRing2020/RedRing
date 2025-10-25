//! Point3D の安全な変換機能
//!
//! Result型を使用した適切なエラーハンドリング実装

use crate::{Angle, Point3D, Scalar, Vector3D};
use geo_foundation::TransformError;

// ============================================================================
// Safe Transform Operations for Point3D
// ============================================================================

impl<T: Scalar> Point3D<T> {
    /// 安全な平行移動
    ///
    /// Point3Dの平行移動は基本的に失敗しないが、値の妥当性チェックを行う
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい点、または適切なエラー
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Point3D<T>, TransformError> {
        // 有限値チェック
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "Translation vector contains non-finite values".to_string(),
            ));
        }

        let new_x = self.x() + translation.x();
        let new_y = self.y() + translation.y();
        let new_z = self.z() + translation.z();

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation result contains non-finite coordinates".to_string(),
            ));
        }

        Ok(Point3D::new(new_x, new_y, new_z))
    }

    /// 安全なスケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点、または適切なエラー
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Point3D<T>, TransformError> {
        // スケール倍率の検証
        if factor == T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor cannot be zero".to_string(),
            ));
        }

        if !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factor must be finite".to_string(),
            ));
        }

        // 中心点の有限値チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale center contains non-finite coordinates".to_string(),
            ));
        }

        let dx = self.x() - center.x();
        let dy = self.y() - center.y();
        let dz = self.z() - center.z();

        let new_x = center.x() + dx * factor;
        let new_y = center.y() + dy * factor;
        let new_z = center.z() + dz * factor;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale transformation resulted in non-finite coordinates".to_string(),
            ));
        }

        Ok(Point3D::new(new_x, new_y, new_z))
    }

    /// 安全な回転変換（Z軸周り）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `axis` - 回転軸ベクトル
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい点、または適切なエラー
    pub fn safe_rotate(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Point3D<T>, TransformError> {
        // 回転軸の検証
        let axis_length_squared = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
        if axis_length_squared == T::ZERO {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be a zero vector".to_string(),
            ));
        }

        // 中心点の有限値チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation center contains non-finite coordinates".to_string(),
            ));
        }

        // 角度の有限値チェック
        if !angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation(
                "Rotation angle must be finite".to_string(),
            ));
        }

        // 簡易実装: Z軸周りの回転のみサポート
        // TODO: 任意軸周りの回転を実装する場合、analysisクレートの行列演算を使用
        let cos_angle = angle.to_radians().cos();
        let sin_angle = angle.to_radians().sin();

        let dx = self.x() - center.x();
        let dy = self.y() - center.y();
        let dz = self.z() - center.z();

        let new_x = center.x() + dx * cos_angle - dy * sin_angle;
        let new_y = center.y() + dx * sin_angle + dy * cos_angle;
        let new_z = center.z() + dz; // Z座標は変化なし（Z軸周り回転）

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation transformation resulted in non-finite coordinates".to_string(),
            ));
        }

        Ok(Point3D::new(new_x, new_y, new_z))
    }

    /// 複合変換の安全な実行
    ///
    /// 平行移動→回転の順で実行
    pub fn safe_translate_and_rotate(
        &self,
        translation: Vector3D<T>,
        rotation_center: Point3D<T>,
        rotation_axis: Vector3D<T>,
        rotation_angle: Angle<T>,
    ) -> Result<Point3D<T>, TransformError> {
        let translated = self.safe_translate(translation)?;
        translated.safe_rotate(rotation_center, rotation_axis, rotation_angle)
    }

    /// スケール→平行移動の安全な実行
    pub fn safe_scale_and_translate(
        &self,
        scale_center: Point3D<T>,
        scale_factor: T,
        translation: Vector3D<T>,
    ) -> Result<Point3D<T>, TransformError> {
        let scaled = self.safe_scale(scale_center, scale_factor)?;
        scaled.safe_translate(translation)
    }

    /// 詳細検証
    ///
    /// 点の有効性を包括的にチェック
    pub fn detailed_validation(&self) -> Result<(), TransformError> {
        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Point contains non-finite coordinates".to_string(),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// 点の値が安全な範囲内かチェック
pub fn validate_point_coordinates<T: Scalar>(point: &Point3D<T>) -> Result<(), TransformError> {
    if !point.x().is_finite() || !point.y().is_finite() || !point.z().is_finite() {
        return Err(TransformError::InvalidGeometry(
            "Point coordinates must be finite".to_string(),
        ));
    }
    Ok(())
}
