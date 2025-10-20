//! Vector3D の安全な変換機能
//!
//! Result型を使用した適切なエラーハンドリング実装
//!
//! ## 設計方針
//!
//! このファイルは実装層として機能し、詳細なエラーハンドリングを提供：
//!
//! - **Result<T, TransformError>**: 明示的なエラー処理
//! - **包括的検証**: 入力値・計算結果の有限性チェック
//! - **詳細エラーメッセージ**: デバッグとトラブルシューティング支援
//! - **複合変換**: エラー伝播を考慮した安全なチェーン処理
//!
//! ## Vector3Dの特徴
//!
//! Vector3DはPoint3Dと異なり、3次元空間での方向と大きさを表現：
//! - **平行移動**: ベクトル加算として実装
//! - **回転**: 原点周りまたは任意軸周りの回転
//! - **スケール**: 原点からのスケールまたは任意中心点から
//! - **軸固有回転**: X, Y, Z軸周りの個別回転
//! - **正規化**: 単位ベクトル化とゼロベクトル検証
//!
//! ## 使用例
//!
//! ```rust
//! use geo_primitives::{Vector3D, Point3D};
//! use geo_foundation::{Angle, TransformError};
//!
//! let vector = Vector3D::new(1.0, 2.0, 3.0);
//! let axis = Vector3D::new(0.0, 0.0, 1.0); // Z軸
//! let rotation_angle = Angle::from_radians(std::f64::consts::PI / 2.0);
//!
//! match vector.safe_rotate_axis_origin(axis, rotation_angle) {
//!     Ok(rotated) => println!("回転成功: {:?}", rotated),
//!     Err(TransformError::ZeroVector(msg)) => {
//!         println!("ゼロベクトルエラー: {}", msg);
//!     },
//!     Err(e) => println!("その他のエラー: {}", e),
//! }
//! ```

use crate::{Angle, Point3D, Scalar, Vector3D};
use geo_foundation::TransformError;

// ============================================================================
// Safe Transform Operations for Vector3D
// ============================================================================

impl<T: Scalar> Vector3D<T> {
    /// 安全な平行移動（ベクトル加算）
    ///
    /// Vector3Dの平行移動はベクトル加算として実装
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しいベクトル、または適切なエラー
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Vector3D<T>, TransformError> {
        // 有限値チェック
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "Translation vector contains non-finite values".to_string(),
            ));
        }

        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite values".to_string(),
            ));
        }

        let new_x = self.x() + translation.x();
        let new_y = self.y() + translation.y();
        let new_z = self.z() + translation.z();

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation result contains non-finite components".to_string(),
            ));
        }

        Ok(Vector3D::new(new_x, new_y, new_z))
    }

    /// 安全な原点からのスケール
    ///
    /// ベクトルを原点からスケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しいベクトル、または適切なエラー
    pub fn safe_scale_origin(&self, factor: T) -> Result<Vector3D<T>, TransformError> {
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

        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        let new_x = self.x() * factor;
        let new_y = self.y() * factor;
        let new_z = self.z() * factor;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale transformation resulted in non-finite components".to_string(),
            ));
        }

        Ok(Vector3D::new(new_x, new_y, new_z))
    }

    /// 安全な任意点からのスケール
    ///
    /// ベクトルを点として扱い、指定した中心点からスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しいベクトル、または適切なエラー
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Vector3D<T>, TransformError> {
        // ベクトルを点として扱ってスケール
        let point = Point3D::new(self.x(), self.y(), self.z());
        let scaled_point = point.safe_scale(center, factor)?;

        Ok(Vector3D::new(
            scaled_point.x(),
            scaled_point.y(),
            scaled_point.z(),
        ))
    }

    /// 安全な任意軸周りの回転（原点中心）
    ///
    /// 指定した軸周りにベクトルを回転
    ///
    /// # 引数
    /// * `axis` - 回転軸ベクトル
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate_axis_origin(
        &self,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Vector3D<T>, TransformError> {
        // 軸ベクトルの検証
        if !axis.x().is_finite() || !axis.y().is_finite() || !axis.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation axis contains non-finite components".to_string(),
            ));
        }

        let axis_length = axis.magnitude();
        if axis_length == T::ZERO {
            return Err(TransformError::ZeroVector(
                "Rotation axis cannot be zero vector".to_string(),
            ));
        }

        if !axis_length.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation axis length is not finite".to_string(),
            ));
        }

        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        // 角度の有限値チェック
        if !angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation(
                "Rotation angle must be finite".to_string(),
            ));
        }

        // 軸を正規化
        let normalized_axis = Vector3D::new(
            axis.x() / axis_length,
            axis.y() / axis_length,
            axis.z() / axis_length,
        );

        // ロドリゲス回転公式の実装
        let cos_angle = angle.to_radians().cos();
        let sin_angle = angle.to_radians().sin();
        let one_minus_cos = T::ONE - cos_angle;

        // k・v (軸と自身の内積)
        let dot_product = normalized_axis.x() * self.x()
            + normalized_axis.y() * self.y()
            + normalized_axis.z() * self.z();

        // k × v (軸と自身の外積)
        let cross_x = normalized_axis.y() * self.z() - normalized_axis.z() * self.y();
        let cross_y = normalized_axis.z() * self.x() - normalized_axis.x() * self.z();
        let cross_z = normalized_axis.x() * self.y() - normalized_axis.y() * self.x();

        // ロドリゲス公式: v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k・v)*(1-cos(θ))
        let new_x = self.x() * cos_angle
            + cross_x * sin_angle
            + normalized_axis.x() * dot_product * one_minus_cos;
        let new_y = self.y() * cos_angle
            + cross_y * sin_angle
            + normalized_axis.y() * dot_product * one_minus_cos;
        let new_z = self.z() * cos_angle
            + cross_z * sin_angle
            + normalized_axis.z() * dot_product * one_minus_cos;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation result contains non-finite components".to_string(),
            ));
        }

        Ok(Vector3D::new(new_x, new_y, new_z))
    }

    /// 安全な任意軸周りの回転（任意中心点）
    ///
    /// ベクトルを点として扱い、指定した中心点・軸周りに回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `axis` - 回転軸ベクトル
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Vector3D<T>, TransformError> {
        // ベクトルを点として扱って回転
        let point = Point3D::new(self.x(), self.y(), self.z());
        let rotated_point = point.safe_rotate(center, axis, angle)?;

        Ok(Vector3D::new(
            rotated_point.x(),
            rotated_point.y(),
            rotated_point.z(),
        ))
    }

    /// 安全なX軸周り回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate_x_origin(&self, angle: Angle<T>) -> Result<Vector3D<T>, TransformError> {
        let x_axis = Vector3D::new(T::ONE, T::ZERO, T::ZERO);
        self.safe_rotate_axis_origin(x_axis, angle)
    }

    /// 安全なY軸周り回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate_y_origin(&self, angle: Angle<T>) -> Result<Vector3D<T>, TransformError> {
        let y_axis = Vector3D::new(T::ZERO, T::ONE, T::ZERO);
        self.safe_rotate_axis_origin(y_axis, angle)
    }

    /// 安全なZ軸周り回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Vector3D<T>, TransformError> {
        let z_axis = Vector3D::new(T::ZERO, T::ZERO, T::ONE);
        self.safe_rotate_axis_origin(z_axis, angle)
    }

    /// 安全な正規化
    ///
    /// ベクトルを単位ベクトルに変換
    ///
    /// # 戻り値
    /// 正規化された新しいベクトル、または適切なエラー
    pub fn safe_normalize(&self) -> Result<Vector3D<T>, TransformError> {
        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        let length = self.magnitude();

        if length == T::ZERO {
            return Err(TransformError::ZeroVector(
                "Cannot normalize zero vector".to_string(),
            ));
        }

        if !length.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Vector length is not finite".to_string(),
            ));
        }

        let new_x = self.x() / length;
        let new_y = self.y() / length;
        let new_z = self.z() / length;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() || !new_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Normalization resulted in non-finite components".to_string(),
            ));
        }

        Ok(Vector3D::new(new_x, new_y, new_z))
    }

    /// 複合変換の安全な実行
    ///
    /// 平行移動→Z軸回転の順で実行
    pub fn safe_translate_and_rotate_z_origin(
        &self,
        translation: Vector3D<T>,
        rotation_angle: Angle<T>,
    ) -> Result<Vector3D<T>, TransformError> {
        let translated = self.safe_translate(translation)?;
        translated.safe_rotate_z_origin(rotation_angle)
    }

    /// スケール→平行移動の安全な実行（原点スケール）
    pub fn safe_scale_and_translate_origin(
        &self,
        scale_factor: T,
        translation: Vector3D<T>,
    ) -> Result<Vector3D<T>, TransformError> {
        let scaled = self.safe_scale_origin(scale_factor)?;
        scaled.safe_translate(translation)
    }

    /// 詳細検証
    ///
    /// ベクトルの有効性を包括的にチェック
    pub fn detailed_validation(&self) -> Result<(), TransformError> {
        if !self.x().is_finite() || !self.y().is_finite() || !self.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Vector contains non-finite components".to_string(),
            ));
        }
        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// ベクトルの値が安全な範囲内かチェック
pub fn validate_vector_components<T: Scalar>(vector: &Vector3D<T>) -> Result<(), TransformError> {
    if !vector.x().is_finite() || !vector.y().is_finite() || !vector.z().is_finite() {
        return Err(TransformError::InvalidGeometry(
            "Vector components must be finite".to_string(),
        ));
    }
    Ok(())
}

/// 回転軸ベクトルの有効性をチェック
pub fn validate_rotation_axis<T: Scalar>(axis: &Vector3D<T>) -> Result<(), TransformError> {
    validate_vector_components(axis)?;

    let length = axis.magnitude();
    if length == T::ZERO {
        return Err(TransformError::ZeroVector(
            "Rotation axis cannot be zero vector".to_string(),
        ));
    }

    if !length.is_finite() {
        return Err(TransformError::InvalidGeometry(
            "Rotation axis length is not finite".to_string(),
        ));
    }

    Ok(())
}
