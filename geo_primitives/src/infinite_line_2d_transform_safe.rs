//! InfiniteLine2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{InfiniteLine2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// InfiniteLine2Dの安全な変換操作
impl<T: Scalar> InfiniteLine2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine2D)` - 移動後の直線
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        let new_point = self.point() + translation;
        Ok(Self::new(new_point, self.direction().as_vector()).unwrap())
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine2D)` - 回転後の直線
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
    /// * `Ok(InfiniteLine2D)` - 回転後の直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate(&self, center: Point2D<T>, angle: Angle<T>) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 直線上の点を回転
        let relative_point = self.point() - center;
        let rotated_point_x = relative_point.x() * cos_angle - relative_point.y() * sin_angle;
        let rotated_point_y = relative_point.x() * sin_angle + relative_point.y() * cos_angle;
        let new_point = center + Vector2D::new(rotated_point_x, rotated_point_y);

        // 方向ベクトルを回転
        let dir = self.direction().as_vector();
        let rotated_dir_x = dir.x() * cos_angle - dir.y() * sin_angle;
        let rotated_dir_y = dir.x() * sin_angle + dir.y() * cos_angle;
        let new_direction = Vector2D::new(rotated_dir_x, rotated_dir_y);

        Ok(Self::new(new_point, new_direction).unwrap())
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine2D)` - スケール後の直線
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
    /// * `Ok(InfiniteLine2D)` - スケール後の直線
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point2D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        // スケール倍率の有効性チェック
        if factor.is_zero() || !factor.is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        let relative_point = Vector2D::from_points(center, self.point());
        let scaled_point = relative_point * factor;
        let new_point = center + scaled_point;

        // 方向ベクトルはスケールされない（無限直線の方向は不変）
        Ok(Self::new(new_point, self.direction().as_vector()).unwrap())
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine2D)` - スケール後の直線
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
    /// * `Ok(InfiniteLine2D)` - スケール後の直線
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale_non_uniform(
        &self,
        center: Point2D<T>,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        // スケール倍率の有効性チェック
        if scale_x.is_zero() || scale_y.is_zero() || !scale_x.is_finite() || !scale_y.is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        let relative_point = Vector2D::from_points(center, self.point());
        let scaled_point =
            Vector2D::new(relative_point.x() * scale_x, relative_point.y() * scale_y);
        let new_point = center + scaled_point;

        // 方向ベクトルも非均一スケールの影響を受ける
        let dir = self.direction().as_vector();
        let scaled_direction = Vector2D::new(dir.x() * scale_x, dir.y() * scale_y);

        // 新しい方向ベクトルで直線を作成
        Self::new(new_point, scaled_direction)
            .ok_or(TransformError::InvalidGeometry("無効な入力".to_string()))
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine2D)` - 反射後の直線
    /// * `Err(TransformError)` - 無効な軸（ゼロベクトル、無限大、NaN）
    pub fn safe_reflect(
        &self,
        axis_point: Point2D<T>,
        axis_direction: Vector2D<T>,
    ) -> Result<Self, TransformError> {
        // 軸上の点の有効性チェック
        if !axis_point.x().is_finite() || !axis_point.y().is_finite() {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        // 軸方向ベクトルの有効性チェック
        if !axis_direction.x().is_finite()
            || !axis_direction.y().is_finite()
            || axis_direction.magnitude().is_zero()
        {
            return Err(TransformError::InvalidGeometry("無効な入力".to_string()));
        }

        // 軸方向を正規化
        let axis_normalized = axis_direction.normalize();

        // 直線上の点を反射
        let to_point = Vector2D::from_points(axis_point, self.point());
        let projection_length = to_point.dot(&axis_normalized);
        let projection = axis_normalized * projection_length;
        let perpendicular = to_point - projection;
        let reflected_point = axis_point + projection - perpendicular;

        // 方向ベクトルを反射
        let dir = self.direction().as_vector();
        let dir_projection_length = dir.dot(&axis_normalized);
        let dir_projection = axis_normalized * dir_projection_length;
        let dir_perpendicular = dir - dir_projection;
        let reflected_direction = dir_projection - dir_perpendicular;

        Ok(Self::new(reflected_point, reflected_direction).unwrap())
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
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

        let translation = Vector2D::new(3.0, 4.0);
        let result = line.safe_translate(translation).unwrap();

        let expected_point = Point2D::new(4.0, 6.0);
        let tolerance = 1e-10;
        assert!((result.point().x() - expected_point.x()).abs() < tolerance);
        assert!((result.point().y() - expected_point.y()).abs() < tolerance);

        // 方向は変わらない
        assert!((result.direction().x() - line.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - line.direction().y()).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = line.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = line.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

        // 90度回転
        let result = line.safe_rotate_origin(Angle::from_degrees(90.0)).unwrap();

        let tolerance = 1e-10;
        // 点 (1,0) が (0,1) に回転
        assert!((result.point().x() - 0.0).abs() < tolerance);
        assert!((result.point().y() - 1.0).abs() < tolerance);

        // 方向 (1,0) が (0,1) に回転
        assert!((result.direction().x() - 0.0).abs() < tolerance);
        assert!((result.direction().y() - 1.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 0.0), Vector2D::new(1.0, 0.0)).unwrap();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = line.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(2.0, 3.0), Vector2D::new(1.0, 0.0)).unwrap();

        let result = line.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 点がスケールされる
        assert!((result.point().x() - 4.0).abs() < tolerance);
        assert!((result.point().y() - 6.0).abs() < tolerance);

        // 方向は変わらない（正規化済み）
        assert!((result.direction().x() - line.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - line.direction().y()).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 2.0), Vector2D::new(1.0, 0.0)).unwrap();

        let result = line.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(2.0, 3.0), Vector2D::new(1.0, 1.0)).unwrap();

        let result = line.safe_scale_non_uniform_origin(2.0, 3.0).unwrap();

        let tolerance = 1e-10;
        // 点が非均一スケールされる
        assert!((result.point().x() - 4.0).abs() < tolerance);
        assert!((result.point().y() - 9.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let line =
            InfiniteLine2D::<f64>::new(Point2D::new(1.0, 1.0), Vector2D::new(1.0, 0.0)).unwrap();

        // X軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(1.0, 0.0);
        let result = line.safe_reflect(axis_point, axis_direction).unwrap();

        let tolerance = 1e-10;
        // 点 (1,1) が (1,-1) に反射
        assert!((result.point().x() - 1.0).abs() < tolerance);
        assert!((result.point().y() - (-1.0)).abs() < tolerance);

        // 方向 (1,0) は変わらない
        assert!((result.direction().x() - 1.0).abs() < tolerance);
        assert!((result.direction().y() - 0.0).abs() < tolerance);
    }
}
