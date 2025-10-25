//! Vector2D の安全な変換機能
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
//! ## Vector2Dの特徴
//!
//! Vector2DはPoint2Dと異なり、方向と大きさを表現：
//! - **平行移動**: ベクトル加算として実装
//! - **回転**: 原点周りの回転が基本（中心点指定も可能）
//! - **スケール**: 原点からのスケールまたは任意中心点から
//! - **正規化**: 単位ベクトル化とゼロベクトル検証
//!
//! ## 使用例
//!
//! ```rust
//! use geo_primitives::{Vector2D, Point2D};
//! use geo_foundation::{Angle, TransformError};
//!
//! let vector = Vector2D::new(3.0, 4.0);
//! let rotation_angle = Angle::from_radians(std::f64::consts::PI / 2.0);
//!
//! match vector.safe_rotate_origin(rotation_angle) {
//!     Ok(rotated) => println!("回転成功: {:?}", rotated),
//!     Err(TransformError::InvalidRotation(msg)) => {
//!         println!("回転エラー: {}", msg);
//!     },
//!     Err(e) => println!("その他のエラー: {}", e),
//! }
//! ```

use crate::{Angle, Point2D, Scalar, Vector2D};
use geo_foundation::TransformError;

// ============================================================================
// Safe Transform Operations for Vector2D
// ============================================================================

impl<T: Scalar> Vector2D<T> {
    /// 安全な平行移動（ベクトル加算）
    ///
    /// Vector2Dの平行移動はベクトル加算として実装
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しいベクトル、または適切なエラー
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Vector2D<T>, TransformError> {
        // 有限値チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation vector contains non-finite values".to_string(),
            ));
        }

        if !self.x().is_finite() || !self.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite values".to_string(),
            ));
        }

        let new_x = self.x() + translation.x();
        let new_y = self.y() + translation.y();

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation result contains non-finite components".to_string(),
            ));
        }

        Ok(Vector2D::new(new_x, new_y))
    }

    /// 安全な原点周り回転
    ///
    /// ベクトルを原点を中心に回転
    ///
    /// # 引数
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate_origin(&self, angle: Angle<T>) -> Result<Vector2D<T>, TransformError> {
        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() {
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

        let cos_angle = angle.to_radians().cos();
        let sin_angle = angle.to_radians().sin();

        let new_x = self.x() * cos_angle - self.y() * sin_angle;
        let new_y = self.x() * sin_angle + self.y() * cos_angle;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation result contains non-finite components".to_string(),
            ));
        }

        Ok(Vector2D::new(new_x, new_y))
    }

    /// 安全な任意点周り回転
    ///
    /// ベクトルを点として扱い、指定した中心点周りに回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しいベクトル、または適切なエラー
    pub fn safe_rotate(
        &self,
        center: Point2D<T>,
        angle: Angle<T>,
    ) -> Result<Vector2D<T>, TransformError> {
        // 中心点の有限値チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation center contains non-finite coordinates".to_string(),
            ));
        }

        // ベクトルを点として扱って回転
        let point = Point2D::new(self.x(), self.y());
        let rotated_point = point.safe_rotate(center, angle)?;

        Ok(Vector2D::new(rotated_point.x(), rotated_point.y()))
    }

    /// 安全な原点スケール
    ///
    /// ベクトルを原点からスケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しいベクトル、または適切なエラー
    pub fn safe_scale_origin(&self, factor: T) -> Result<Vector2D<T>, TransformError> {
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
        if !self.x().is_finite() || !self.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        let new_x = self.x() * factor;
        let new_y = self.y() * factor;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale transformation resulted in non-finite components".to_string(),
            ));
        }

        Ok(Vector2D::new(new_x, new_y))
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
    pub fn safe_scale(&self, center: Point2D<T>, factor: T) -> Result<Vector2D<T>, TransformError> {
        // ベクトルを点として扱ってスケール
        let point = Point2D::new(self.x(), self.y());
        let scaled_point = point.safe_scale(center, factor)?;

        Ok(Vector2D::new(scaled_point.x(), scaled_point.y()))
    }

    /// 安全な非等方スケール（原点から）
    ///
    /// # 引数
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しいベクトル、または適切なエラー
    pub fn safe_scale_non_uniform_origin(
        &self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Vector2D<T>, TransformError> {
        // スケール倍率の検証
        if scale_x == T::ZERO || scale_y == T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors cannot be zero".to_string(),
            ));
        }

        if !scale_x.is_finite() || !scale_y.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "Scale factors must be finite".to_string(),
            ));
        }

        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        let new_x = self.x() * scale_x;
        let new_y = self.y() * scale_y;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Non-uniform scale transformation resulted in non-finite components".to_string(),
            ));
        }

        Ok(Vector2D::new(new_x, new_y))
    }

    /// 安全な正規化
    ///
    /// ベクトルを単位ベクトルに変換
    ///
    /// # 戻り値
    /// 正規化された新しいベクトル、または適切なエラー
    pub fn safe_normalize(&self) -> Result<Vector2D<T>, TransformError> {
        // 自身の有限値チェック
        if !self.x().is_finite() || !self.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Source vector contains non-finite components".to_string(),
            ));
        }

        let length = self.magnitude();

        if length == T::ZERO {
            return Err(TransformError::InvalidGeometry(
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

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Normalization resulted in non-finite components".to_string(),
            ));
        }

        Ok(Vector2D::new(new_x, new_y))
    }

    /// 複合変換の安全な実行
    ///
    /// 平行移動→回転の順で実行
    pub fn safe_translate_and_rotate_origin(
        &self,
        translation: Vector2D<T>,
        rotation_angle: Angle<T>,
    ) -> Result<Vector2D<T>, TransformError> {
        let translated = self.safe_translate(translation)?;
        translated.safe_rotate_origin(rotation_angle)
    }

    /// スケール→平行移動の安全な実行（原点スケール）
    pub fn safe_scale_and_translate_origin(
        &self,
        scale_factor: T,
        translation: Vector2D<T>,
    ) -> Result<Vector2D<T>, TransformError> {
        let scaled = self.safe_scale_origin(scale_factor)?;
        scaled.safe_translate(translation)
    }

    /// 詳細検証
    ///
    /// ベクトルの有効性を包括的にチェック
    pub fn detailed_validation(&self) -> Result<(), TransformError> {
        if !self.x().is_finite() || !self.y().is_finite() {
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
pub fn validate_vector_components<T: Scalar>(vector: &Vector2D<T>) -> Result<(), TransformError> {
    if !vector.x().is_finite() || !vector.y().is_finite() {
        return Err(TransformError::InvalidGeometry(
            "Vector components must be finite".to_string(),
        ));
    }
    Ok(())
}
