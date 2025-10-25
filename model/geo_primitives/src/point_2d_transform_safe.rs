//! Point2D の安全な変換機能
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
//! ## 使用場面
//!
//! - ユーザー入力の検証が必要
//! - 数値計算の精度が重要
//! - エラー原因の詳細な分析が必要
//! - 批評的なアプリケーション（CAD/CAM等）
//!
//! ## 使用例
//!
//! ```rust
//! use geo_primitives::{Point2D, Vector2D};
//! use geo_foundation::{Angle, TransformError};
//!
//! let point = Point2D::new(1.0, 2.0);
//! let translation = Vector2D::new(f64::INFINITY, 0.0); // 無効な入力
//!
//! match point.safe_translate(translation) {
//!     Ok(result) => println!("変換成功: {:?}", result),
//!     Err(TransformError::InvalidGeometry(msg)) => {
//!         println!("幾何エラー: {}", msg);
//!     },
//!     Err(e) => println!("その他のエラー: {}", e),
//! }
//! ```

use crate::{Angle, Point2D, Scalar, Vector2D};
use geo_foundation::TransformError;

// ============================================================================
// Safe Transform Operations for Point2D
// ============================================================================

impl<T: Scalar> Point2D<T> {
    /// 安全な平行移動
    ///
    /// Point2Dの平行移動は基本的に失敗しないが、値の妥当性チェックを行う
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい点、または適切なエラー
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Point2D<T>, TransformError> {
        // 有限値チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation vector contains non-finite values".to_string(),
            ));
        }

        let new_x = self.x() + translation.x();
        let new_y = self.y() + translation.y();

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Translation result contains non-finite coordinates".to_string(),
            ));
        }

        Ok(Point2D::new(new_x, new_y))
    }

    /// 安全なスケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点、または適切なエラー
    pub fn safe_scale(&self, center: Point2D<T>, factor: T) -> Result<Point2D<T>, TransformError> {
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
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale center contains non-finite coordinates".to_string(),
            ));
        }

        let dx = self.x() - center.x();
        let dy = self.y() - center.y();

        let new_x = center.x() + dx * factor;
        let new_y = center.y() + dy * factor;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale transformation resulted in non-finite coordinates".to_string(),
            ));
        }

        Ok(Point2D::new(new_x, new_y))
    }

    /// 安全な非等方スケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点、または適切なエラー
    pub fn safe_scale_non_uniform(
        &self,
        center: Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Point2D<T>, TransformError> {
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

        // 中心点の有限値チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Scale center contains non-finite coordinates".to_string(),
            ));
        }

        let dx = self.x() - center.x();
        let dy = self.y() - center.y();

        let new_x = center.x() + dx * scale_x;
        let new_y = center.y() + dy * scale_y;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Non-uniform scale transformation resulted in non-finite coordinates".to_string(),
            ));
        }

        Ok(Point2D::new(new_x, new_y))
    }

    /// 安全な回転変換
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい点、または適切なエラー
    pub fn safe_rotate(
        &self,
        center: Point2D<T>,
        angle: Angle<T>,
    ) -> Result<Point2D<T>, TransformError> {
        // 中心点の有限値チェック
        if !center.x().is_finite() || !center.y().is_finite() {
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

        let cos_angle = angle.to_radians().cos();
        let sin_angle = angle.to_radians().sin();

        let dx = self.x() - center.x();
        let dy = self.y() - center.y();

        let new_x = center.x() + dx * cos_angle - dy * sin_angle;
        let new_y = center.y() + dx * sin_angle + dy * cos_angle;

        // 結果の有限値チェック
        if !new_x.is_finite() || !new_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rotation transformation resulted in non-finite coordinates".to_string(),
            ));
        }

        Ok(Point2D::new(new_x, new_y))
    }

    /// 複合変換の安全な実行
    ///
    /// 平行移動→回転の順で実行
    pub fn safe_translate_and_rotate(
        &self,
        translation: Vector2D<T>,
        rotation_center: Point2D<T>,
        rotation_angle: Angle<T>,
    ) -> Result<Point2D<T>, TransformError> {
        let translated = self.safe_translate(translation)?;
        translated.safe_rotate(rotation_center, rotation_angle)
    }

    /// スケール→平行移動の安全な実行
    pub fn safe_scale_and_translate(
        &self,
        scale_center: Point2D<T>,
        scale_factor: T,
        translation: Vector2D<T>,
    ) -> Result<Point2D<T>, TransformError> {
        let scaled = self.safe_scale(scale_center, scale_factor)?;
        scaled.safe_translate(translation)
    }

    /// 詳細検証
    ///
    /// 点の有効性を包括的にチェック
    pub fn detailed_validation(&self) -> Result<(), TransformError> {
        if !self.x().is_finite() || !self.y().is_finite() {
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
pub fn validate_point_coordinates<T: Scalar>(point: &Point2D<T>) -> Result<(), TransformError> {
    if !point.x().is_finite() || !point.y().is_finite() {
        return Err(TransformError::InvalidGeometry(
            "Point coordinates must be finite".to_string(),
        ));
    }
    Ok(())
}
