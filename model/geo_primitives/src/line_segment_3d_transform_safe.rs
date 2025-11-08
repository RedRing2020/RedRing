//! LineSegment3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{LineSegment3D, Point3D, Vector3D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// LineSegment3Dの安全な変換操作
impl<T: Scalar> LineSegment3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 移動後の線分
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

        let new_start = self.start() + translation;
        let new_end = self.end() + translation;

        // 移動結果の有効性チェック
        if !new_start.x().is_finite()
            || !new_start.y().is_finite()
            || !new_start.z().is_finite()
            || !new_end.x().is_finite()
            || !new_end.y().is_finite()
            || !new_end.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "移動計算結果が無効".to_string(),
            ));
        }

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("移動後の線分が縮退".to_string()))
    }

    /// 安全な回転（Z軸中心、原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 回転後の線分
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_z(Point3D::origin(), angle)
    }

    /// 安全な回転（Z軸中心、指定点中心）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 回転後の線分
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

        // 始点をZ軸回転
        let new_start = Self::rotate_point_z(self.start(), center, cos_angle, sin_angle)?;

        // 終点をZ軸回転
        let new_end = Self::rotate_point_z(self.end(), center, cos_angle, sin_angle)?;

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("回転後の線分が縮退".to_string()))
    }

    /// 安全な任意軸回転（Rodriguesの公式使用）
    ///
    /// # 引数
    /// * `axis_point` - 回転軸上の点
    /// * `axis_direction` - 回転軸の方向ベクトル
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 回転後の線分
    /// * `Err(TransformError)` - 無効な入力（ゼロ軸、無限大、NaN）
    pub fn safe_rotate_axis(
        &self,
        axis_point: Point3D<T>,
        axis_direction: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 軸上の点の有効性チェック
        if !axis_point.x().is_finite() || !axis_point.y().is_finite() || !axis_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な軸上の点".to_string(),
            ));
        }

        // 軸方向ベクトルの有効性チェック
        if !axis_direction.x().is_finite()
            || !axis_direction.y().is_finite()
            || !axis_direction.z().is_finite()
            || axis_direction.magnitude().is_zero()
        {
            return Err(TransformError::ZeroVector(
                "無効な軸方向ベクトル".to_string(),
            ));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        // 軸方向を正規化
        let axis_normalized = axis_direction.normalize();

        // 始点を軸回転
        let new_start =
            Self::rotate_point_rodrigues(self.start(), axis_point, axis_normalized, angle_rad)?;

        // 終点を軸回転
        let new_end =
            Self::rotate_point_rodrigues(self.end(), axis_point, axis_normalized, angle_rad)?;

        Self::new(new_start, new_end).ok_or(TransformError::ZeroVector(
            "任意軸回転後の線分が縮退".to_string(),
        ))
    }

    /// 3D点のZ軸回転計算
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `center` - 回転中心点
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point_z(
        point: Point3D<T>,
        center: Point3D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Point3D<T>, TransformError> {
        let relative = point - center;
        let rotated_x = relative.x() * cos_angle - relative.y() * sin_angle;
        let rotated_y = relative.x() * sin_angle + relative.y() * cos_angle;
        let rotated_z = relative.z(); // Z座標は変化なし

        // 回転結果の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() || !rotated_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Z軸回転計算結果が無効".to_string(),
            ));
        }

        let result = center + Vector3D::new(rotated_x, rotated_y, rotated_z);
        Ok(result)
    }

    /// Rodriguesの公式による任意軸回転
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `axis_point` - 回転軸上の点
    /// * `axis_normalized` - 正規化された回転軸ベクトル
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point_rodrigues(
        point: Point3D<T>,
        axis_point: Point3D<T>,
        axis_normalized: Vector3D<T>,
        angle: T,
    ) -> Result<Point3D<T>, TransformError> {
        let v = point - axis_point;
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Rodriguesの公式: v' = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_cross_v = axis_normalized.cross(&v);
        let k_dot_v = axis_normalized.dot(&v);

        let one = T::from_f64(1.0);
        let rotated_v =
            v * cos_angle + k_cross_v * sin_angle + axis_normalized * (k_dot_v * (one - cos_angle));

        // 回転結果の有効性チェック
        if !rotated_v.x().is_finite() || !rotated_v.y().is_finite() || !rotated_v.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rodrigues回転計算結果が無効".to_string(),
            ));
        }

        let result = axis_point + rotated_v;
        Ok(result)
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - スケール後の線分
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
    /// * `Ok(LineSegment3D)` - スケール後の線分
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

        let scaled_start = center + (self.start() - center) * factor;
        let scaled_end = center + (self.end() - center) * factor;

        // スケール結果の有効性チェック
        if !scaled_start.x().is_finite()
            || !scaled_start.y().is_finite()
            || !scaled_start.z().is_finite()
            || !scaled_end.x().is_finite()
            || !scaled_end.y().is_finite()
            || !scaled_end.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(scaled_start, scaled_end).ok_or(TransformError::ZeroVector(
            "スケール後の線分が縮退".to_string(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効なスケール倍率（0、無限大、NaN）
    pub fn safe_scale_non_uniform_origin(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        self.safe_scale_non_uniform(Point3D::origin(), scale_x, scale_y, scale_z)
    }

    /// 安全な非均一スケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効な入力（0倍率、無限大、NaN）
    pub fn safe_scale_non_uniform(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if scale_x.is_zero()
            || scale_y.is_zero()
            || scale_z.is_zero()
            || !scale_x.is_finite()
            || !scale_y.is_finite()
            || !scale_z.is_finite()
        {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        // 始点の非均一スケール
        let start_relative = self.start() - center;
        let scaled_start_relative = Vector3D::new(
            start_relative.x() * scale_x,
            start_relative.y() * scale_y,
            start_relative.z() * scale_z,
        );
        let scaled_start = center + scaled_start_relative;

        // 終点の非均一スケール
        let end_relative = self.end() - center;
        let scaled_end_relative = Vector3D::new(
            end_relative.x() * scale_x,
            end_relative.y() * scale_y,
            end_relative.z() * scale_z,
        );
        let scaled_end = center + scaled_end_relative;

        // スケール結果の有効性チェック
        if !scaled_start.x().is_finite()
            || !scaled_start.y().is_finite()
            || !scaled_start.z().is_finite()
            || !scaled_end.x().is_finite()
            || !scaled_end.y().is_finite()
            || !scaled_end.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "非均一スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(scaled_start, scaled_end).ok_or(TransformError::ZeroVector(
            "非均一スケール後の線分が縮退".to_string(),
        ))
    }

    /// 安全な反射（平面に対する）
    ///
    /// # 引数
    /// * `plane_point` - 反射平面上の点
    /// * `plane_normal` - 反射平面の法線ベクトル
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 反射後の線分
    /// * `Err(TransformError)` - 無効な平面（ゼロ法線、無限大、NaN）
    pub fn safe_reflect(
        &self,
        plane_point: Point3D<T>,
        plane_normal: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        // 平面上の点の有効性チェック
        if !plane_point.x().is_finite()
            || !plane_point.y().is_finite()
            || !plane_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な平面上の点".to_string(),
            ));
        }

        // 平面法線ベクトルの有効性チェック
        if !plane_normal.x().is_finite()
            || !plane_normal.y().is_finite()
            || !plane_normal.z().is_finite()
            || plane_normal.magnitude().is_zero()
        {
            return Err(TransformError::ZeroVector(
                "無効な平面法線ベクトル".to_string(),
            ));
        }

        // 法線を正規化
        let normal_normalized = plane_normal.normalize();

        // 始点を反射
        let reflected_start = Self::reflect_point(self.start(), plane_point, normal_normalized)?;

        // 終点を反射
        let reflected_end = Self::reflect_point(self.end(), plane_point, normal_normalized)?;

        Self::new(reflected_start, reflected_end)
            .ok_or(TransformError::ZeroVector("反射後の線分が縮退".to_string()))
    }

    /// 点を平面に対して反射
    ///
    /// # 引数
    /// * `point` - 反射対象の点
    /// * `plane_point` - 反射平面上の点
    /// * `normal_normalized` - 正規化された平面法線ベクトル
    ///
    /// # 戻り値
    /// 反射後の点
    fn reflect_point(
        point: Point3D<T>,
        plane_point: Point3D<T>,
        normal_normalized: Vector3D<T>,
    ) -> Result<Point3D<T>, TransformError> {
        let to_point = point - plane_point;
        let distance_to_plane = to_point.dot(&normal_normalized);
        let two = T::from_f64(2.0);
        let reflected_point = point - normal_normalized * (distance_to_plane * two);

        // 反射結果の有効性チェック
        if !reflected_point.x().is_finite()
            || !reflected_point.y().is_finite()
            || !reflected_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "反射計算結果が無効".to_string(),
            ));
        }

        Ok(reflected_point)
    }

    /// 安全な線分の延長
    ///
    /// # 引数
    /// * `start_extension` - 始点側の延長長さ
    /// * `end_extension` - 終点側の延長長さ
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - 延長後の線分
    /// * `Err(TransformError)` - 無効な延長長さ（無限大、NaN）
    pub fn safe_extend(
        &self,
        start_extension: T,
        end_extension: T,
    ) -> Result<Self, TransformError> {
        // 延長長さの有効性チェック
        if !start_extension.is_finite() || !end_extension.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な延長長さ".to_string(),
            ));
        }

        let direction = self.direction();
        let new_start = self.start() - direction * start_extension;
        let new_end = self.end() + direction * end_extension;

        // 延長結果の有効性チェック
        if !new_start.x().is_finite()
            || !new_start.y().is_finite()
            || !new_start.z().is_finite()
            || !new_end.x().is_finite()
            || !new_end.y().is_finite()
            || !new_end.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "延長計算結果が無効".to_string(),
            ));
        }

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("延長後の線分が縮退".to_string()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_segment() -> LineSegment3D<f64> {
        LineSegment3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0)).unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let segment = create_test_segment();
        let translation = Vector3D::new(2.0, 3.0, 4.0);
        let result = segment.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.start().x() - 3.0).abs() < tolerance);
        assert!((result.start().y() - 5.0).abs() < tolerance);
        assert!((result.start().z() - 7.0).abs() < tolerance);
        assert!((result.end().x() - 6.0).abs() < tolerance);
        assert!((result.end().y() - 8.0).abs() < tolerance);
        assert!((result.end().z() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let segment = create_test_segment();

        // 無限大の移動ベクトル
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let result = segment.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector3D::new(f64::NAN, 0.0, 0.0);
        let result = segment.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 5.0), Point3D::new(2.0, 0.0, 5.0)).unwrap();

        // 90度Z軸回転
        let result = segment
            .safe_rotate_z_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // (1,0,5) が (0,1,5) に、(2,0,5) が (0,2,5) に回転
        assert!((result.start().x() - 0.0).abs() < tolerance);
        assert!((result.start().y() - 1.0).abs() < tolerance);
        assert!((result.start().z() - 5.0).abs() < tolerance);
        assert!((result.end().x() - 0.0).abs() < tolerance);
        assert!((result.end().y() - 2.0).abs() < tolerance);
        assert!((result.end().z() - 5.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_axis_success() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();

        // Z軸中心90度回転
        let axis_point = Point3D::origin();
        let axis_direction = Vector3D::new(0.0, 0.0, 1.0);
        let result = segment
            .safe_rotate_axis(axis_point, axis_direction, Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // X軸上の点がY軸上に回転
        assert!((result.start().x() - 0.0).abs() < tolerance);
        assert!((result.start().y() - 1.0).abs() < tolerance);
        assert!((result.start().z() - 0.0).abs() < tolerance);
        assert!((result.end().x() - 0.0).abs() < tolerance);
        assert!((result.end().y() - 2.0).abs() < tolerance);
        assert!((result.end().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let segment = create_test_segment();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = segment.safe_rotate_z_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let segment = create_test_segment();
        let result = segment.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 原点中心でのスケールなので座標が倍になる
        assert!((result.start().x() - 2.0).abs() < tolerance);
        assert!((result.start().y() - 4.0).abs() < tolerance);
        assert!((result.start().z() - 6.0).abs() < tolerance);
        assert!((result.end().x() - 8.0).abs() < tolerance);
        assert!((result.end().y() - 10.0).abs() < tolerance);
        assert!((result.end().z() - 12.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let segment = create_test_segment();
        let result = segment.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let segment = create_test_segment();
        let result = segment
            .safe_scale_non_uniform_origin(2.0, 3.0, 4.0)
            .unwrap();

        let tolerance = 1e-10;
        // X方向に2倍、Y方向に3倍、Z方向に4倍のスケール
        assert!((result.start().x() - 2.0).abs() < tolerance);
        assert!((result.start().y() - 6.0).abs() < tolerance);
        assert!((result.start().z() - 12.0).abs() < tolerance);
        assert!((result.end().x() - 8.0).abs() < tolerance);
        assert!((result.end().y() - 15.0).abs() < tolerance);
        assert!((result.end().z() - 24.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(2.0, 2.0, 2.0)).unwrap();

        // XY平面に対する反射（Z座標が反転）
        let plane_point = Point3D::origin();
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let result = segment.safe_reflect(plane_point, plane_normal).unwrap();

        let tolerance = 1e-10;
        // Z座標が反転される
        assert!((result.start().x() - 1.0).abs() < tolerance);
        assert!((result.start().y() - 1.0).abs() < tolerance);
        assert!((result.start().z() - (-1.0)).abs() < tolerance);
        assert!((result.end().x() - 2.0).abs() < tolerance);
        assert!((result.end().y() - 2.0).abs() < tolerance);
        assert!((result.end().z() - (-2.0)).abs() < tolerance);
    }

    #[test]
    fn test_safe_extend_success() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();

        let result = segment.safe_extend(1.0, 1.0).unwrap();

        let tolerance = 1e-10;
        // 始点が-1、終点が3になる（X方向）
        assert!((result.start().x() - (-1.0)).abs() < tolerance);
        assert!((result.start().y() - 0.0).abs() < tolerance);
        assert!((result.start().z() - 0.0).abs() < tolerance);
        assert!((result.end().x() - 3.0).abs() < tolerance);
        assert!((result.end().y() - 0.0).abs() < tolerance);
        assert!((result.end().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_zero_normal_error() {
        let segment = create_test_segment();
        let plane_point = Point3D::origin();
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);
        let result = segment.safe_reflect(plane_point, zero_normal);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_axis_zero_axis_error() {
        let segment = create_test_segment();
        let axis_point = Point3D::origin();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let result = segment.safe_rotate_axis(axis_point, zero_axis, Angle::from_degrees(90.0));
        assert!(result.is_err());
    }
}

/// LineSegment3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> LineSegment3D<T> {
    /// トレランス制約付きスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の線分の長さがトレランス以下
    pub fn safe_scale_with_tolerance(&self, factor: T) -> Result<Self, TransformError> {
        self.safe_scale_with_tolerance_center(Point3D::origin(), factor)
    }

    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment3D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の線分の長さがトレランス以下
    pub fn safe_scale_with_tolerance_center(
        &self,
        center: Point3D<T>,
        factor: T,
    ) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心点です".to_string(),
            ));
        }

        let new_length = self.length() * factor;

        // 長さの幾何学的制約チェック（トレランスベース）
        let min_length = T::DISTANCE_TOLERANCE;
        if new_length <= min_length {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の線分長({:?})がトレランス({:?})以下になります",
                new_length, min_length
            )));
        }

        // 基本のスケール処理を実行
        self.safe_scale(center, factor)
    }

    /// 長さスケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// この線分に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_length = T::DISTANCE_TOLERANCE;
        let current_length = self.length();

        if current_length <= T::ZERO {
            T::ZERO
        } else {
            // 最小長さを維持するための倍率
            // スケール後長さ = 現在長さ × 倍率 >= トレランス
            // 最小倍率 = トレランス / 現在長さ
            min_length / current_length
        }
    }

    /// スケール倍率の事前検証
    ///
    /// # 引数
    /// * `factor` - チェックするスケール倍率
    ///
    /// # 戻り値
    /// スケール倍率が有効かどうか
    pub fn validate_scale_factor(&self, factor: T) -> bool {
        if factor <= T::ZERO || !factor.is_finite() {
            return false;
        }

        let new_length = self.length() * factor;
        let min_length = T::DISTANCE_TOLERANCE;

        new_length >= min_length && new_length.is_finite()
    }
}
