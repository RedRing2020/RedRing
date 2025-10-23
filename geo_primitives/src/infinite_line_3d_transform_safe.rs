//! InfiniteLine3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{InfiniteLine3D, Point3D, Vector3D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// InfiniteLine3Dの安全な変換操作
impl<T: Scalar> InfiniteLine3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine3D)` - 移動後の直線
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        let new_point = self.point() + translation;
        Ok(Self::new(new_point, self.direction().as_vector()).unwrap())
    }

    /// 安全なZ軸周り回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine3D)` - 回転後の直線
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_z(Point3D::origin(), angle)
    }

    /// 安全なZ軸周り回転（指定点中心）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine3D)` - 回転後の直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate_z(
        &self,
        center: Point3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な回転中心".to_string(),
            ));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 直線上の点をZ軸周りに回転
        let relative_point = self.point() - center;
        let new_x = relative_point.x() * cos_angle - relative_point.y() * sin_angle;
        let new_y = relative_point.x() * sin_angle + relative_point.y() * cos_angle;
        let new_point = center + Vector3D::new(new_x, new_y, relative_point.z());

        // 方向ベクトルをZ軸周りに回転
        let dir = self.direction().as_vector();
        let new_dir_x = dir.x() * cos_angle - dir.y() * sin_angle;
        let new_dir_y = dir.x() * sin_angle + dir.y() * cos_angle;
        let new_direction = Vector3D::new(new_dir_x, new_dir_y, dir.z());

        Ok(Self::new(new_point, new_direction).unwrap())
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine3D)` - スケール後の直線
    /// * `Err(TransformError)` - 無効なスケール倍率（0、無限大、NaN）
    pub fn safe_scale_origin(&self, factor: T) -> Result<Self, TransformError> {
        self.safe_scale(Point3D::origin(), factor)
    }

    /// 安全なスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(InfiniteLine3D)` - スケール後の直線
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
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

        let relative_point = Vector3D::from_points(&center, &self.point());
        let scaled_point = relative_point * factor;
        let new_point = center + scaled_point;

        // 方向ベクトルはスケールされない（無限直線の方向は不変）
        Ok(Self::new(new_point, self.direction().as_vector()).unwrap())
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
            InfiniteLine3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        let translation = Vector3D::new(3.0, 4.0, 5.0);
        let result = line.safe_translate(translation).unwrap();

        let expected_point = Point3D::new(4.0, 6.0, 8.0);
        let tolerance = 1e-10;
        assert!((result.point().x() - expected_point.x()).abs() < tolerance);
        assert!((result.point().y() - expected_point.y()).abs() < tolerance);
        assert!((result.point().z() - expected_point.z()).abs() < tolerance);

        // 方向は変わらない
        assert!((result.direction().x() - line.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - line.direction().y()).abs() < tolerance);
        assert!((result.direction().z() - line.direction().z()).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let line =
            InfiniteLine3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        // 無限大の移動ベクトル
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let result = line.safe_translate(invalid_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let line =
            InfiniteLine3D::<f64>::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        // Z軸周りに90度回転
        let result = line
            .safe_rotate_z_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // 点 (1,0,0) が (0,1,0) に回転
        assert!((result.point().x() - 0.0).abs() < tolerance);
        assert!((result.point().y() - 1.0).abs() < tolerance);
        assert!((result.point().z() - 0.0).abs() < tolerance);

        // 方向 (1,0,0) が (0,1,0) に回転
        assert!((result.direction().x() - 0.0).abs() < tolerance);
        assert!((result.direction().y() - 1.0).abs() < tolerance);
        assert!((result.direction().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let line =
            InfiniteLine3D::<f64>::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = line.safe_rotate_z_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let line =
            InfiniteLine3D::<f64>::new(Point3D::new(2.0, 3.0, 4.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        let result = line.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 点がスケールされる
        assert!((result.point().x() - 4.0).abs() < tolerance);
        assert!((result.point().y() - 6.0).abs() < tolerance);
        assert!((result.point().z() - 8.0).abs() < tolerance);

        // 方向は変わらない（正規化済み）
        assert!((result.direction().x() - line.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - line.direction().y()).abs() < tolerance);
        assert!((result.direction().z() - line.direction().z()).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let line =
            InfiniteLine3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0))
                .unwrap();

        let result = line.safe_scale_origin(0.0);
        assert!(result.is_err());
    }
}
