//! LineSegment2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{LineSegment2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// LineSegment2Dの安全な変換操作
impl<T: Scalar> LineSegment2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - 移動後の線分
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        let new_start = self.start_point() + translation;
        let new_end = self.end_point() + translation;

        // 移動結果の有効性チェック
        if !new_start.x().is_finite()
            || !new_start.y().is_finite()
            || !new_end.x().is_finite()
            || !new_end.y().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "移動計算結果が無効".to_string(),
            ));
        }

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("移動後の線分が縮退".to_string()))
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - 回転後の線分
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
    /// * `Ok(LineSegment2D)` - 回転後の線分
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

        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 始点を回転
        let start_relative = self.start_point() - center;
        let rotated_start = Self::rotate_point(start_relative, cos_angle, sin_angle)?;
        let new_start = center + rotated_start;

        // 終点を回転
        let end_relative = self.end_point() - center;
        let rotated_end = Self::rotate_point(end_relative, cos_angle, sin_angle)?;
        let new_end = center + rotated_end;

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("回転後の線分が縮退".to_string()))
    }

    /// 2D点の回転計算
    ///
    /// # 引数
    /// * `point` - 回転対象の相対位置ベクトル
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後の相対位置ベクトル
    fn rotate_point(
        point: Vector2D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Vector2D<T>, TransformError> {
        let rotated_x = point.x() * cos_angle - point.y() * sin_angle;
        let rotated_y = point.x() * sin_angle + point.y() * cos_angle;

        // 回転結果の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        Ok(Vector2D::new(rotated_x, rotated_y))
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - スケール後の線分
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
    /// * `Ok(LineSegment2D)` - スケール後の線分
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

        let scaled_start = center + (self.start_point() - center) * factor;
        let scaled_end = center + (self.end_point() - center) * factor;

        // スケール結果の有効性チェック
        if !scaled_start.x().is_finite()
            || !scaled_start.y().is_finite()
            || !scaled_end.x().is_finite()
            || !scaled_end.y().is_finite()
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
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - スケール後の線分
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
    /// * `Ok(LineSegment2D)` - スケール後の線分
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

        // 始点の非均一スケール
        let start_relative = self.start_point() - center;
        let scaled_start_relative =
            Vector2D::new(start_relative.x() * scale_x, start_relative.y() * scale_y);
        let scaled_start = center + scaled_start_relative;

        // 終点の非均一スケール
        let end_relative = self.end_point() - center;
        let scaled_end_relative =
            Vector2D::new(end_relative.x() * scale_x, end_relative.y() * scale_y);
        let scaled_end = center + scaled_end_relative;

        // スケール結果の有効性チェック
        if !scaled_start.x().is_finite()
            || !scaled_start.y().is_finite()
            || !scaled_end.x().is_finite()
            || !scaled_end.y().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "非均一スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(scaled_start, scaled_end).ok_or(TransformError::ZeroVector(
            "非均一スケール後の線分が縮退".to_string(),
        ))
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - 反射後の線分
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

        // 始点を反射
        let reflected_start = Self::reflect_point(self.start_point(), axis_point, axis_normalized)?;

        // 終点を反射
        let reflected_end = Self::reflect_point(self.end_point(), axis_point, axis_normalized)?;

        Self::new(reflected_start, reflected_end)
            .ok_or(TransformError::ZeroVector("反射後の線分が縮退".to_string()))
    }

    /// 点を軸に対して反射
    ///
    /// # 引数
    /// * `point` - 反射対象の点
    /// * `axis_point` - 反射軸上の点
    /// * `axis_normalized` - 正規化された反射軸方向ベクトル
    ///
    /// # 戻り値
    /// 反射後の点
    fn reflect_point(
        point: Point2D<T>,
        axis_point: Point2D<T>,
        axis_normalized: Vector2D<T>,
    ) -> Result<Point2D<T>, TransformError> {
        let to_point = point - axis_point;
        let projection_length = to_point.dot(&axis_normalized);
        let projection = axis_normalized * projection_length;
        let perpendicular = to_point - projection;
        let reflected_point = axis_point + projection - perpendicular;

        // 反射結果の有効性チェック
        if !reflected_point.x().is_finite() || !reflected_point.y().is_finite() {
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
    /// * `Ok(LineSegment2D)` - 延長後の線分
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
        let new_start = self.start_point() - direction * start_extension;
        let new_end = self.end_point() + direction * end_extension;

        // 延長結果の有効性チェック
        if !new_start.x().is_finite()
            || !new_start.y().is_finite()
            || !new_end.x().is_finite()
            || !new_end.y().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "延長計算結果が無効".to_string(),
            ));
        }

        Self::new(new_start, new_end)
            .ok_or(TransformError::ZeroVector("延長後の線分が縮退".to_string()))
    }
}

/// LineSegment2D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> LineSegment2D<T> {
    /// トレランス制約付きスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の線分の長さがトレランス以下
    pub fn safe_scale_with_tolerance(&self, factor: T) -> Result<Self, TransformError> {
        self.safe_scale_with_tolerance_center(Point2D::origin(), factor)
    }

    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(LineSegment2D)` - スケール後の線分
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の線分の長さがトレランス以下
    pub fn safe_scale_with_tolerance_center(
        &self,
        center: Point2D<T>,
        factor: T,
    ) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_segment() -> LineSegment2D<f64> {
        LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(3.0, 4.0)).unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let segment = create_test_segment();
        let translation = Vector2D::new(2.0, 3.0);
        let result = segment.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.start_point().x() - 3.0).abs() < tolerance);
        assert!((result.start_point().y() - 5.0).abs() < tolerance);
        assert!((result.end_point().x() - 5.0).abs() < tolerance);
        assert!((result.end_point().y() - 7.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let segment = create_test_segment();

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = segment.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = segment.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        // 90度回転
        let result = segment
            .safe_rotate_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // (1,0) が (0,1) に、(2,0) が (0,2) に回転
        assert!((result.start_point().x() - 0.0).abs() < tolerance);
        assert!((result.start_point().y() - 1.0).abs() < tolerance);
        assert!((result.end_point().x() - 0.0).abs() < tolerance);
        assert!((result.end_point().y() - 2.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let segment = create_test_segment();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = segment.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let segment = create_test_segment();
        let result = segment.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 原点中心でのスケールなので座標が倍になる
        assert!((result.start_point().x() - 2.0).abs() < tolerance);
        assert!((result.start_point().y() - 4.0).abs() < tolerance);
        assert!((result.end_point().x() - 6.0).abs() < tolerance);
        assert!((result.end_point().y() - 8.0).abs() < tolerance);
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
        let result = segment.safe_scale_non_uniform_origin(2.0, 3.0).unwrap();

        let tolerance = 1e-10;
        // X方向に2倍、Y方向に3倍のスケール
        assert!((result.start_point().x() - 2.0).abs() < tolerance);
        assert!((result.start_point().y() - 6.0).abs() < tolerance);
        assert!((result.end_point().x() - 6.0).abs() < tolerance);
        assert!((result.end_point().y() - 12.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 1.0), Point2D::new(2.0, 2.0)).unwrap();

        // X軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(1.0, 0.0);
        let result = segment.safe_reflect(axis_point, axis_direction).unwrap();

        let tolerance = 1e-10;
        // Y座標が反転される
        assert!((result.start_point().x() - 1.0).abs() < tolerance);
        assert!((result.start_point().y() - (-1.0)).abs() < tolerance);
        assert!((result.end_point().x() - 2.0).abs() < tolerance);
        assert!((result.end_point().y() - (-2.0)).abs() < tolerance);
    }

    #[test]
    fn test_safe_extend_success() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let result = segment.safe_extend(1.0, 1.0).unwrap();

        let tolerance = 1e-10;
        // 始点が-1、終点が3になる
        assert!((result.start_point().x() - (-1.0)).abs() < tolerance);
        assert!((result.start_point().y() - 0.0).abs() < tolerance);
        assert!((result.end_point().x() - 3.0).abs() < tolerance);
        assert!((result.end_point().y() - 0.0).abs() < tolerance);
    }
}
