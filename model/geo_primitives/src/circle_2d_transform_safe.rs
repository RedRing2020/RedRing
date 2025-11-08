//! Circle2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Circle2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// Circle2Dの安全な変換操作
impl<T: Scalar> Circle2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 移動後の円
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        let new_center = self.center() + translation;

        // 移動結果の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "移動計算結果が無効".to_string(),
            ));
        }

        Self::new(new_center, self.radius()).ok_or(TransformError::InvalidGeometry(
            "移動後の円の作成に失敗".to_string(),
        ))
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 回転後の円
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
    /// * `Ok(Circle2D)` - 回転後の円
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

        // 中心点を回転
        let rotated_center = Self::rotate_point(self.center(), center, cos_angle, sin_angle)?;

        Self::new(rotated_center, self.radius()).ok_or(TransformError::InvalidGeometry(
            "回転後の円の作成に失敗".to_string(),
        ))
    }

    /// 2D点の回転計算
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `center` - 回転中心点
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point(
        point: Point2D<T>,
        center: Point2D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Point2D<T>, TransformError> {
        let relative = point - center;
        let rotated_x = relative.x() * cos_angle - relative.y() * sin_angle;
        let rotated_y = relative.x() * sin_angle + relative.y() * cos_angle;

        // 回転結果の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        let result = center + Vector2D::new(rotated_x, rotated_y);
        Ok(result)
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - スケール後の円
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
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
    /// * `Ok(Circle2D)` - スケール後の円
    /// * `Err(TransformError)` - 無効な入力（0以下倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point2D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        let scaled_center = center + (self.center() - center) * factor;
        let scaled_radius = self.radius() * factor;

        // スケール後の半径がゼロ以下でないことを確認（基本チェック）
        if scaled_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "スケール後の半径がゼロ以下になります".to_string(),
            ));
        }

        // スケール結果の有効性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_radius.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(scaled_center, scaled_radius).ok_or(TransformError::InvalidGeometry(
            "スケール後の円の作成に失敗".to_string(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// 非均一スケールは円を楕円に変換するため、このメソッドは意図的にエラーを返す
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Err(TransformError)` - 円は非均一スケールできない
    pub fn safe_scale_non_uniform_origin(
        &self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        self.safe_scale_non_uniform(Point2D::origin(), scale_x, scale_y)
    }

    /// 安全な非均一スケール（指定点中心）
    ///
    /// 非均一スケールは円を楕円に変換するため、均一スケールのみサポート
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - X/Yスケールが等しい場合のみ
    /// * `Err(TransformError)` - 非均一スケールまたは無効な入力
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
        if scale_x <= T::ZERO || scale_y <= T::ZERO || !scale_x.is_finite() || !scale_y.is_finite()
        {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        // 均一スケールのみ許可（円を保持）
        let tolerance = T::from_f64(1e-10);
        if (scale_x - scale_y).abs() > tolerance {
            return Err(TransformError::InvalidGeometry(
                "円の非均一スケールは楕円になるため非対応".to_string(),
            ));
        }

        // 均一スケールとして処理
        self.safe_scale(center, scale_x)
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 反射後の円
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

        // 中心点を反射
        let reflected_center = Self::reflect_point(self.center(), axis_point, axis_normalized)?;

        Self::new(reflected_center, self.radius()).ok_or(TransformError::InvalidGeometry(
            "反射後の円の作成に失敗".to_string(),
        ))
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

    /// 安全な半径のみスケール（中心固定）
    ///
    /// # 引数
    /// * `factor` - 半径のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 半径スケール後の円
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_radius(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な半径スケール倍率".to_string(),
            ));
        }

        let new_radius = self.radius() * factor;

        // スケール結果の有効性チェック
        if !new_radius.is_finite() || new_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "半径スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(self.center(), new_radius).ok_or(TransformError::InvalidGeometry(
            "半径スケール後の円の作成に失敗".to_string(),
        ))
    }

    /// 安全な中心点変更（半径固定）
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 新しい中心の円
    /// * `Err(TransformError)` - 無効な中心点（無限大、NaN）
    pub fn safe_with_center(&self, new_center: Point2D<T>) -> Result<Self, TransformError> {
        // 新しい中心点の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な新しい中心点".to_string(),
            ));
        }

        Self::new(new_center, self.radius()).ok_or(TransformError::InvalidGeometry(
            "新しい中心での円の作成に失敗".to_string(),
        ))
    }

    /// 安全な半径変更（中心固定）
    ///
    /// # 引数
    /// * `new_radius` - 新しい半径
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - 新しい半径の円
    /// * `Err(TransformError)` - 無効な半径（0以下、無限大、NaN）
    pub fn safe_with_radius(&self, new_radius: T) -> Result<Self, TransformError> {
        // 新しい半径の有効性チェック
        if new_radius <= T::ZERO || !new_radius.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な新しい半径".to_string(),
            ));
        }

        Self::new(self.center(), new_radius).ok_or(TransformError::InvalidGeometry(
            "新しい半径での円の作成に失敗".to_string(),
        ))
    }
}

/// Circle2D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> Circle2D<T> {
    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(Circle2D)` - スケール後の円
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の半径がトレランス以下
    pub fn safe_scale_with_tolerance(
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

        let scaled_center = center + (self.center() - center) * factor;
        let scaled_radius = self.radius() * factor;

        // 半径の幾何学的制約チェック（トレランスベース）
        let min_radius = T::DISTANCE_TOLERANCE;
        if scaled_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径({:?})がトレランス({:?})以下になります",
                scaled_radius, min_radius
            )));
        }

        // 数値安定性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_radius.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        Self::new(scaled_center, scaled_radius).ok_or(TransformError::InvalidGeometry(
            "スケール後の円の作成に失敗しました".to_string(),
        ))
    }

    /// 半径の最小許容スケール倍率を取得
    ///
    /// # 戻り値
    /// この円に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_radius = T::DISTANCE_TOLERANCE;
        let current_radius = self.radius();

        if current_radius <= T::ZERO {
            T::ZERO
        } else {
            // 最小半径を維持するための倍率 + 安全マージン
            let min_factor = min_radius / current_radius;
            min_factor + T::DISTANCE_TOLERANCE
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_circle() -> Circle2D<f64> {
        Circle2D::new(Point2D::new(2.0, 3.0), 5.0).unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let circle = create_test_circle();
        let translation = Vector2D::new(3.0, 4.0);
        let result = circle.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.center().x() - 5.0).abs() < tolerance);
        assert!((result.center().y() - 7.0).abs() < tolerance);
        assert!((result.radius() - 5.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let circle = create_test_circle();

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = circle.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = circle.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let circle = Circle2D::new(Point2D::new(3.0, 0.0), 2.0).unwrap();

        // 90度回転
        let result = circle
            .safe_rotate_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // (3,0) が (0,3) に回転
        assert!((result.center().x() - 0.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.radius() - 2.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let circle = create_test_circle();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = circle.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let circle = create_test_circle();
        let result = circle.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 中心と半径がともに2倍
        assert!((result.center().x() - 4.0).abs() < tolerance);
        assert!((result.center().y() - 6.0).abs() < tolerance);
        assert!((result.radius() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let circle = create_test_circle();
        let result = circle.safe_scale_origin(0.0);
        assert!(result.is_err());

        let result = circle.safe_scale_origin(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_uniform_case() {
        let circle = create_test_circle();
        // 同じスケール値なら成功
        let result = circle.safe_scale_non_uniform_origin(2.0, 2.0).unwrap();

        let tolerance = 1e-10;
        assert!((result.center().x() - 4.0).abs() < tolerance);
        assert!((result.center().y() - 6.0).abs() < tolerance);
        assert!((result.radius() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_non_uniform_error() {
        let circle = create_test_circle();
        // 異なるスケール値はエラー
        let result = circle.safe_scale_non_uniform_origin(2.0, 3.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_reflect_success() {
        let circle = Circle2D::new(Point2D::new(3.0, 2.0), 1.5).unwrap();

        // X軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(1.0, 0.0);
        let result = circle.safe_reflect(axis_point, axis_direction).unwrap();

        let tolerance = 1e-10;
        // Y座標が反転される
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - (-2.0)).abs() < tolerance);
        assert!((result.radius() - 1.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_radius_success() {
        let circle = create_test_circle();
        let result = circle.safe_scale_radius(1.5).unwrap();

        let tolerance = 1e-10;
        // 中心は変わらず、半径のみ1.5倍
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.radius() - 7.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_center_success() {
        let circle = create_test_circle();
        let new_center = Point2D::new(10.0, 15.0);
        let result = circle.safe_with_center(new_center).unwrap();

        let tolerance = 1e-10;
        // 半径は変わらず、中心のみ変更
        assert!((result.center().x() - 10.0).abs() < tolerance);
        assert!((result.center().y() - 15.0).abs() < tolerance);
        assert!((result.radius() - 5.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_radius_success() {
        let circle = create_test_circle();
        let result = circle.safe_with_radius(8.0).unwrap();

        let tolerance = 1e-10;
        // 中心は変わらず、半径のみ変更
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.radius() - 8.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_radius_invalid() {
        let circle = create_test_circle();

        // 0以下の半径はエラー
        let result = circle.safe_with_radius(0.0);
        assert!(result.is_err());

        let result = circle.safe_with_radius(-1.0);
        assert!(result.is_err());

        // 無限大の半径はエラー
        let result = circle.safe_with_radius(f64::INFINITY);
        assert!(result.is_err());
    }
}
