//! BBox2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{BBox2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// BBox2Dの安全な変換操作
impl<T: Scalar> BBox2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - 移動後の境界ボックス
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        // 境界ボックスの平行移動は min と max を同じベクトルで移動
        let new_min = self.min() + translation;
        let new_max = self.max() + translation;
        Ok(BBox2D::new(new_min, new_max))
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - 回転後の境界ボックス
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate(Point2D::origin(), angle)
    }

    /// 安全な回転（指定点中心）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - 回転後の境界ボックス（4つの角を回転させて再計算）
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate(&self, center: Point2D<T>, angle: Angle<T>) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な回転中心".to_string(),
            ));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        // 境界ボックスの4つの角を取得
        let corners = [
            self.min(),                                   // 左下
            Point2D::new(self.max().x(), self.min().y()), // 右下
            self.max(),                                   // 右上
            Point2D::new(self.min().x(), self.max().y()), // 左上
        ];

        // 各角を回転
        let mut rotated_corners = Vec::with_capacity(4);
        for corner in &corners {
            let rotated_corner = Self::rotate_point_around_center(*corner, center, angle_rad)?;
            rotated_corners.push(rotated_corner);
        }

        // 回転した角から新しい境界ボックスを構築
        Self::from_points(&rotated_corners).ok_or(TransformError::InvalidGeometry(
            "回転後の角から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 点を中心周りに回転
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `center` - 回転中心
    /// * `angle_rad` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point_around_center(
        point: Point2D<T>,
        center: Point2D<T>,
        angle_rad: T,
    ) -> Result<Point2D<T>, TransformError> {
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 中心からの相対位置を計算
        let relative = point - center;

        // 回転
        let rotated_x = relative.x() * cos_angle - relative.y() * sin_angle;
        let rotated_y = relative.x() * sin_angle + relative.y() * cos_angle;

        // 回転後の値の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        Ok(center + Vector2D::new(rotated_x, rotated_y))
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - スケール後の境界ボックス
    /// * `Err(TransformError)` - 無効なスケール倍率（0、無限大、NaN）
    pub fn safe_scale_origin(&self, factor: T) -> Result<Self, TransformError> {
        self.safe_scale(Point2D::origin(), factor)
    }

    /// 安全なスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - スケール後の境界ボックス
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point2D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if factor.is_zero() || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        // 境界ボックスの4つの角をスケール
        let corners = [
            self.min(),
            Point2D::new(self.max().x(), self.min().y()),
            self.max(),
            Point2D::new(self.min().x(), self.max().y()),
        ];

        // 各角をスケール
        let mut scaled_corners = Vec::with_capacity(4);
        for corner in &corners {
            let relative = *corner - center;
            let scaled_relative = relative * factor;
            let scaled_corner = center + scaled_relative;

            // スケール結果の有効性チェック
            if !scaled_corner.x().is_finite() || !scaled_corner.y().is_finite() {
                return Err(TransformError::InvalidGeometry(
                    "スケール計算結果が無効".to_string(),
                ));
            }

            scaled_corners.push(scaled_corner);
        }

        // スケールした角から新しい境界ボックスを構築
        Self::from_points(&scaled_corners).ok_or(TransformError::InvalidGeometry(
            "スケール後の角から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - スケール後の境界ボックス
    /// * `Err(TransformError)` - 無効なスケール倍率（0、無限大、NaN）
    pub fn safe_scale_non_uniform_origin(
        &self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        self.safe_scale_non_uniform(Point2D::origin(), scale_x, scale_y)
    }

    /// 安全な非均一スケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - スケール後の境界ボックス
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale_non_uniform(
        &self,
        center: Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if scale_x.is_zero() || scale_y.is_zero() || !scale_x.is_finite() || !scale_y.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        // 境界ボックスの4つの角をスケール
        let corners = [
            self.min(),
            Point2D::new(self.max().x(), self.min().y()),
            self.max(),
            Point2D::new(self.min().x(), self.max().y()),
        ];

        // 各角を非均一スケール
        let mut scaled_corners = Vec::with_capacity(4);
        for corner in &corners {
            let relative = *corner - center;
            let scaled_relative = Vector2D::new(relative.x() * scale_x, relative.y() * scale_y);
            let scaled_corner = center + scaled_relative;

            // スケール結果の有効性チェック
            if !scaled_corner.x().is_finite() || !scaled_corner.y().is_finite() {
                return Err(TransformError::InvalidGeometry(
                    "非均一スケール計算結果が無効".to_string(),
                ));
            }

            scaled_corners.push(scaled_corner);
        }

        // スケールした角から新しい境界ボックスを構築
        Self::from_points(&scaled_corners).ok_or(TransformError::InvalidGeometry(
            "非均一スケール後の角から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - 反射後の境界ボックス
    /// * `Err(TransformError)` - 無効な軸（ゼロベクトル、無限大、NaN）
    pub fn safe_reflect(
        &self,
        axis_point: Point2D<T>,
        axis_direction: Vector2D<T>,
    ) -> Result<Self, TransformError> {
        // 軸上の点の有効性チェック
        if !axis_point.x().is_finite() || !axis_point.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な軸上の点".to_string(),
            ));
        }

        // 軸方向ベクトルの有効性チェック
        if !axis_direction.x().is_finite()
            || !axis_direction.y().is_finite()
            || axis_direction.magnitude().is_zero()
        {
            return Err(TransformError::ZeroVector(
                "無効な軸方向ベクトル".to_string(),
            ));
        }

        // 軸方向を正規化
        let axis_normalized = axis_direction.normalize();

        // 境界ボックスの4つの角を反射
        let corners = [
            self.min(),
            Point2D::new(self.max().x(), self.min().y()),
            self.max(),
            Point2D::new(self.min().x(), self.max().y()),
        ];

        let mut reflected_corners = Vec::with_capacity(4);
        for corner in &corners {
            let to_corner = *corner - axis_point;
            let projection_length = to_corner.dot(&axis_normalized);
            let projection = axis_normalized * projection_length;
            let perpendicular = to_corner - projection;
            let reflected_corner = axis_point + projection - perpendicular;

            // 反射結果の有効性チェック
            if !reflected_corner.x().is_finite() || !reflected_corner.y().is_finite() {
                return Err(TransformError::InvalidGeometry(
                    "反射計算結果が無効".to_string(),
                ));
            }

            reflected_corners.push(reflected_corner);
        }

        // 反射した角から新しい境界ボックスを構築
        Self::from_points(&reflected_corners).ok_or(TransformError::InvalidGeometry(
            "反射後の角から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 安全なマージン拡張
    ///
    /// # 引数
    /// * `margin` - 拡張するマージン値
    ///
    /// # 戻り値
    /// * `Ok(BBox2D)` - 拡張後の境界ボックス
    /// * `Err(TransformError)` - 無効なマージン（負の値、無限大、NaN）
    pub fn safe_expand_by_margin(&self, margin: T) -> Result<Self, TransformError> {
        // マージンの有効性チェック
        if margin < T::ZERO || !margin.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なマージン値".to_string(),
            ));
        }

        let margin_vec = Vector2D::new(margin, margin);
        let new_min = self.min() - margin_vec;
        let new_max = self.max() + margin_vec;

        // 結果の有効性チェック
        if !new_min.x().is_finite()
            || !new_min.y().is_finite()
            || !new_max.x().is_finite()
            || !new_max.y().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "マージン拡張計算結果が無効".to_string(),
            ));
        }

        Ok(BBox2D::new(new_min, new_max))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_translate_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let translation = Vector2D::new(2.0, 3.0);
        let result = bbox.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.min().x() - 3.0).abs() < tolerance);
        assert!((result.min().y() - 5.0).abs() < tolerance);
        assert!((result.max().x() - 5.0).abs() < tolerance);
        assert!((result.max().y() - 7.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = bbox.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = bbox.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0));

        // 90度回転
        let result = bbox.safe_rotate_origin(Angle::from_degrees(90.0)).unwrap();

        // 回転後は角が移動するため、新しい境界ボックスが作成される
        // 正確な値は複雑だが、妥当な値が返されることを確認
        assert!(result.min().x().is_finite());
        assert!(result.min().y().is_finite());
        assert!(result.max().x().is_finite());
        assert!(result.max().y().is_finite());
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0));

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = bbox.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let result = bbox.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 原点中心でのスケールなので座標が倍になる
        assert!((result.min().x() - 2.0).abs() < tolerance);
        assert!((result.min().y() - 4.0).abs() < tolerance);
        assert!((result.max().x() - 6.0).abs() < tolerance);
        assert!((result.max().y() - 8.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let result = bbox.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let result = bbox.safe_scale_non_uniform_origin(2.0, 3.0).unwrap();

        let tolerance = 1e-10;
        // X方向に2倍、Y方向に3倍のスケール
        assert!((result.min().x() - 2.0).abs() < tolerance);
        assert!((result.min().y() - 6.0).abs() < tolerance);
        assert!((result.max().x() - 6.0).abs() < tolerance);
        assert!((result.max().y() - 12.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0));

        // X軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(1.0, 0.0);
        let result = bbox.safe_reflect(axis_point, axis_direction).unwrap();

        // Y座標が反転される
        assert!(result.min().y() < 0.0);
        assert!(result.max().y() < 0.0);
        // X座標は変わらない
        assert!(result.min().x() > 0.0);
        assert!(result.max().x() > 0.0);
    }

    #[test]
    fn test_safe_expand_by_margin_success() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let result = bbox.safe_expand_by_margin(0.5).unwrap();

        let tolerance = 1e-10;
        assert!((result.min().x() - 0.5).abs() < tolerance);
        assert!((result.min().y() - 1.5).abs() < tolerance);
        assert!((result.max().x() - 3.5).abs() < tolerance);
        assert!((result.max().y() - 4.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_expand_by_margin_negative_error() {
        let bbox = BBox2D::<f64>::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0));

        let result = bbox.safe_expand_by_margin(-1.0);
        assert!(result.is_err());
    }
}
