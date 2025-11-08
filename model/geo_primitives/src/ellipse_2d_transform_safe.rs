//! Ellipse2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Ellipse2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// Ellipse2Dの安全な変換操作
impl<T: Scalar> Ellipse2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 移動後の楕円
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

        Self::new(
            new_center,
            self.semi_major(),
            self.semi_minor(),
            self.rotation(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "移動後の楕円の作成に失敗".to_string(),
        ))
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 回転後の楕円
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
    /// * `Ok(Ellipse2D)` - 回転後の楕円
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

        // 楕円の回転角を更新
        let new_rotation = self.rotation() + angle_rad;

        Self::new(
            rotated_center,
            self.semi_major(),
            self.semi_minor(),
            new_rotation,
        )
        .ok_or(TransformError::InvalidGeometry(
            "回転後の楕円の作成に失敗".to_string(),
        ))
    }

    /// 点の回転計算
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
    /// * `Ok(Ellipse2D)` - スケール後の楕円
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
    /// * `Ok(Ellipse2D)` - スケール後の楕円
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
        let scaled_semi_major = self.semi_major() * factor;
        let scaled_semi_minor = self.semi_minor() * factor;

        // スケール結果の有効性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_semi_major.is_finite()
            || !scaled_semi_minor.is_finite()
            || scaled_semi_major <= T::ZERO
            || scaled_semi_minor <= T::ZERO
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(
            scaled_center,
            scaled_semi_major,
            scaled_semi_minor,
            self.rotation(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の楕円の作成に失敗".to_string(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_non_uniform_origin(
        &self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self, TransformError> {
        self.safe_scale_non_uniform(Point2D::origin(), scale_x, scale_y)
    }

    /// 安全な非均一スケール（指定点中心）
    ///
    /// 楕円の非均一スケールは複雑な変換を伴うため、軸方向の変換を含む
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - スケール後の楕円
    /// * `Err(TransformError)` - 無効な入力（0以下倍率、無限大、NaN）
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

        // 中心点をスケール
        let scaled_center = center
            + Vector2D::new(
                (self.center().x() - center.x()) * scale_x,
                (self.center().y() - center.y()) * scale_y,
            );

        // 楕円の軸方向ベクトルを計算
        let rotation = self.rotation();
        let cos_rot = rotation.cos();
        let sin_rot = rotation.sin();

        // 長軸方向ベクトル
        let major_axis_vec = Vector2D::new(cos_rot, sin_rot);
        // 短軸方向ベクトル
        let minor_axis_vec = Vector2D::new(-sin_rot, cos_rot);

        // 軸ベクトルに非均一スケールを適用
        let scaled_major_vec =
            Vector2D::new(major_axis_vec.x() * scale_x, major_axis_vec.y() * scale_y);
        let scaled_minor_vec =
            Vector2D::new(minor_axis_vec.x() * scale_x, minor_axis_vec.y() * scale_y);

        // スケール後の軸長を計算
        let scaled_major_length = scaled_major_vec.magnitude() * self.semi_major();
        let scaled_minor_length = scaled_minor_vec.magnitude() * self.semi_minor();

        // スケール後の軸長の有効性チェック
        if !scaled_major_length.is_finite()
            || !scaled_minor_length.is_finite()
            || scaled_major_length <= T::ZERO
            || scaled_minor_length <= T::ZERO
        {
            return Err(TransformError::InvalidGeometry(
                "非均一スケール計算結果が無効".to_string(),
            ));
        }

        // 新しい回転角を計算
        let new_rotation = scaled_major_vec.y().atan2(scaled_major_vec.x());

        // 長軸・短軸の関係を確認し、必要に応じて交換
        let (final_major, final_minor, final_rotation) =
            if scaled_major_length >= scaled_minor_length {
                (scaled_major_length, scaled_minor_length, new_rotation)
            } else {
                // 短軸が長軸より長い場合は軸を交換
                let pi_half = T::from_f64(std::f64::consts::FRAC_PI_2);
                (
                    scaled_minor_length,
                    scaled_major_length,
                    new_rotation + pi_half,
                )
            };

        Self::new(scaled_center, final_major, final_minor, final_rotation).ok_or(
            TransformError::InvalidGeometry("非均一スケール後の楕円の作成に失敗".to_string()),
        )
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 反射後の楕円
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

        // 楕円の回転角も反射による変換を適用
        // 反射は向きを逆転させるため、回転角の調整が必要
        let axis_angle = axis_normalized.y().atan2(axis_normalized.x());
        let two = T::from_f64(2.0);
        let reflected_rotation = two * axis_angle - self.rotation();

        Self::new(
            reflected_center,
            self.semi_major(),
            self.semi_minor(),
            reflected_rotation,
        )
        .ok_or(TransformError::InvalidGeometry(
            "反射後の楕円の作成に失敗".to_string(),
        ))
    }

    /// 点を軸に対して反射
    ///
    /// # 引数
    /// * `point` - 反射対象の点
    /// * `axis_point` - 反射軸上の点
    /// * `axis_normalized` - 正規化された軸方向ベクトル
    ///
    /// # 戻り値
    /// 反射後の点
    fn reflect_point(
        point: Point2D<T>,
        axis_point: Point2D<T>,
        axis_normalized: Vector2D<T>,
    ) -> Result<Point2D<T>, TransformError> {
        let to_point = point - axis_point;

        // 軸に垂直な方向ベクトル
        let axis_normal = Vector2D::new(-axis_normalized.y(), axis_normalized.x());

        // 軸方向成分と垂直成分に分解
        let axis_component = axis_normalized * to_point.dot(&axis_normalized);
        let normal_component = axis_normal * to_point.dot(&axis_normal);

        // 垂直成分を反転して反射点を計算
        let reflected_point = axis_point + axis_component - normal_component;

        // 反射結果の有効性チェック
        if !reflected_point.x().is_finite() || !reflected_point.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "反射計算結果が無効".to_string(),
            ));
        }

        Ok(reflected_point)
    }

    /// 安全な長半軸のみスケール（中心・短軸・回転固定）
    ///
    /// # 引数
    /// * `factor` - 長半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 長半軸スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_major(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な長半軸スケール倍率".to_string(),
            ));
        }

        let new_semi_major = self.semi_major() * factor;

        // スケール結果の有効性チェック
        if !new_semi_major.is_finite() || new_semi_major <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "長半軸スケール計算結果が無効".to_string(),
            ));
        }

        // 長軸・短軸の関係を維持するかチェック
        let final_major = new_semi_major.max(self.semi_minor());
        let final_minor = new_semi_major.min(self.semi_minor());
        let final_rotation = if new_semi_major >= self.semi_minor() {
            self.rotation()
        } else {
            // 軸が交換される場合は90度回転
            self.rotation() + T::from_f64(std::f64::consts::FRAC_PI_2)
        };

        Self::new(self.center(), final_major, final_minor, final_rotation).ok_or(
            TransformError::InvalidGeometry("長半軸スケール後の楕円の作成に失敗".to_string()),
        )
    }

    /// 安全な短半軸のみスケール（中心・長軸・回転固定）
    ///
    /// # 引数
    /// * `factor` - 短半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 短半軸スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_minor(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な短半軸スケール倍率".to_string(),
            ));
        }

        let new_semi_minor = self.semi_minor() * factor;

        // スケール結果の有効性チェック
        if !new_semi_minor.is_finite() || new_semi_minor <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "短半軸スケール計算結果が無効".to_string(),
            ));
        }

        // 長軸・短軸の関係を維持するかチェック
        let final_major = self.semi_major().max(new_semi_minor);
        let final_minor = self.semi_major().min(new_semi_minor);
        let final_rotation = if self.semi_major() >= new_semi_minor {
            self.rotation()
        } else {
            // 軸が交換される場合は90度回転
            self.rotation() + T::from_f64(std::f64::consts::FRAC_PI_2)
        };

        Self::new(self.center(), final_major, final_minor, final_rotation).ok_or(
            TransformError::InvalidGeometry("短半軸スケール後の楕円の作成に失敗".to_string()),
        )
    }

    /// 安全な回転角変更（中心・軸長固定）
    ///
    /// # 引数
    /// * `new_rotation` - 新しい回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 新しい回転角の楕円
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_with_rotation(&self, new_rotation: Angle<T>) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        let rotation_rad = new_rotation.to_radians();
        if !rotation_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        Self::new(
            self.center(),
            self.semi_major(),
            self.semi_minor(),
            rotation_rad,
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい回転角での楕円の作成に失敗".to_string(),
        ))
    }

    /// 安全な中心点変更（軸長・回転固定）
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 新しい中心の楕円
    /// * `Err(TransformError)` - 無効な中心点（無限大、NaN）
    pub fn safe_with_center(&self, new_center: Point2D<T>) -> Result<Self, TransformError> {
        // 新しい中心点の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な新しい中心点".to_string(),
            ));
        }

        Self::new(
            new_center,
            self.semi_major(),
            self.semi_minor(),
            self.rotation(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい中心での楕円の作成に失敗".to_string(),
        ))
    }

    /// 安全な軸長変更（中心・回転固定）
    ///
    /// # 引数
    /// * `new_semi_major` - 新しい長半軸
    /// * `new_semi_minor` - 新しい短半軸
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - 新しい軸長の楕円
    /// * `Err(TransformError)` - 無効な軸長（0以下、無限大、NaN、長軸<短軸）
    pub fn safe_with_axes(
        &self,
        new_semi_major: T,
        new_semi_minor: T,
    ) -> Result<Self, TransformError> {
        // 軸長の有効性チェック
        if new_semi_major <= T::ZERO
            || new_semi_minor <= T::ZERO
            || !new_semi_major.is_finite()
            || !new_semi_minor.is_finite()
            || new_semi_major < new_semi_minor
        {
            return Err(TransformError::InvalidGeometry("無効な軸長".to_string()));
        }

        Self::new(
            self.center(),
            new_semi_major,
            new_semi_minor,
            self.rotation(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい軸長での楕円の作成に失敗".to_string(),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_ellipse() -> Ellipse2D<f64> {
        Ellipse2D::new(
            Point2D::new(3.0, 4.0),
            5.0,                         // 長半軸
            3.0,                         // 短半軸
            std::f64::consts::FRAC_PI_4, // 45度回転
        )
        .unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let ellipse = create_test_ellipse();
        let translation = Vector2D::new(2.0, -1.0);
        let result = ellipse.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.center().x() - 5.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.semi_major() - 5.0).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
        assert!((result.rotation() - std::f64::consts::FRAC_PI_4).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let ellipse = create_test_ellipse();

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = ellipse.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = ellipse.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let center = Point2D::new(4.0, 0.0);
        let ellipse = Ellipse2D::new(center, 2.0, 1.0, 0.0).unwrap();

        // 90度回転
        let result = ellipse
            .safe_rotate_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // (4,0) が (0,4) に回転
        assert!((result.center().x() - 0.0).abs() < tolerance);
        assert!((result.center().y() - 4.0).abs() < tolerance);
        // 軸長は変わらない
        assert!((result.semi_major() - 2.0).abs() < tolerance);
        assert!((result.semi_minor() - 1.0).abs() < tolerance);
        // 回転角が90度増加
        assert!((result.rotation() - std::f64::consts::FRAC_PI_2).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let ellipse = create_test_ellipse();
        let result = ellipse.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 中心と軸長がともに2倍
        assert!((result.center().x() - 6.0).abs() < tolerance);
        assert!((result.center().y() - 8.0).abs() < tolerance);
        assert!((result.semi_major() - 10.0).abs() < tolerance);
        assert!((result.semi_minor() - 6.0).abs() < tolerance);
        // 回転角は変わらない
        assert!((result.rotation() - std::f64::consts::FRAC_PI_4).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let ellipse = create_test_ellipse();
        let result = ellipse.safe_scale_origin(0.0);
        assert!(result.is_err());

        let result = ellipse.safe_scale_origin(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let center = Point2D::new(1.0, 1.0);
        let ellipse = Ellipse2D::new(center, 3.0, 2.0, 0.0).unwrap(); // 軸が座標軸に平行

        let result = ellipse.safe_scale_non_uniform_origin(2.0, 1.5).unwrap();

        let tolerance = 1e-10;
        // 中心がスケール
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 1.5).abs() < tolerance);

        // 軸長も変化（非均一スケールの効果）
        // X方向に2倍、Y方向に1.5倍なので、長軸は6.0、短軸は3.0になる
        assert!((result.semi_major() - 6.0).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let center = Point2D::new(2.0, 3.0);
        let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).unwrap();

        // Y軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(0.0, 1.0);
        let result = ellipse.safe_reflect(axis_point, axis_direction).unwrap();

        let tolerance = 1e-10;
        // X座標が反転される
        assert!((result.center().x() - (-2.0)).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.semi_major() - 4.0).abs() < tolerance);
        assert!((result.semi_minor() - 2.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_semi_major_success() {
        let ellipse = create_test_ellipse();
        let result = ellipse.safe_scale_semi_major(1.5).unwrap();

        let tolerance = 1e-10;
        // 中心・短軸・回転は変わらず、長半軸のみ1.5倍
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - 4.0).abs() < tolerance);
        assert!((result.semi_major() - 7.5).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
        assert!((result.rotation() - std::f64::consts::FRAC_PI_4).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_semi_minor_axis_swap() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 3.0, 2.0, 0.0).unwrap();

        // 短軸を5倍にすると長軸より長くなる（3.0 vs 10.0）
        let result = ellipse.safe_scale_semi_minor(5.0).unwrap();

        let tolerance = 1e-10;
        // 軸が交換されて長軸が10.0、短軸が3.0になる
        assert!((result.semi_major() - 10.0).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
        // 軸が交換されるため90度回転が追加される
        assert!((result.rotation() - std::f64::consts::FRAC_PI_2).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_rotation_success() {
        let ellipse = create_test_ellipse();
        let result = ellipse
            .safe_with_rotation(Angle::from_degrees(60.0))
            .unwrap();

        let tolerance = 1e-10;
        // 中心・軸長は変わらず、回転角のみ変更
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - 4.0).abs() < tolerance);
        assert!((result.semi_major() - 5.0).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
        assert!((result.rotation() - std::f64::consts::PI / 3.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_center_success() {
        let ellipse = create_test_ellipse();
        let new_center = Point2D::new(10.0, -5.0);
        let result = ellipse.safe_with_center(new_center).unwrap();

        let tolerance = 1e-10;
        // 軸長・回転は変わらず、中心のみ変更
        assert!((result.center().x() - 10.0).abs() < tolerance);
        assert!((result.center().y() - (-5.0)).abs() < tolerance);
        assert!((result.semi_major() - 5.0).abs() < tolerance);
        assert!((result.semi_minor() - 3.0).abs() < tolerance);
        assert!((result.rotation() - std::f64::consts::FRAC_PI_4).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_axes_success() {
        let ellipse = create_test_ellipse();
        let result = ellipse.safe_with_axes(6.0, 4.0).unwrap();

        let tolerance = 1e-10;
        // 中心・回転は変わらず、軸長のみ変更
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - 4.0).abs() < tolerance);
        assert!((result.semi_major() - 6.0).abs() < tolerance);
        assert!((result.semi_minor() - 4.0).abs() < tolerance);
        assert!((result.rotation() - std::f64::consts::FRAC_PI_4).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_axes_invalid_error() {
        let ellipse = create_test_ellipse();

        // 長軸 < 短軸はエラー
        let result = ellipse.safe_with_axes(2.0, 5.0);
        assert!(result.is_err());

        // ゼロ軸長はエラー
        let result = ellipse.safe_with_axes(0.0, 3.0);
        assert!(result.is_err());

        // 負の軸長はエラー
        let result = ellipse.safe_with_axes(5.0, -3.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_reflect_zero_axis_error() {
        let ellipse = create_test_ellipse();
        let axis_point = Point2D::origin();
        let zero_axis = Vector2D::new(0.0, 0.0);
        let result = ellipse.safe_reflect(axis_point, zero_axis);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let ellipse = create_test_ellipse();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = ellipse.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }
}

/// Ellipse2D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> Ellipse2D<T> {
    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の半軸長がトレランス以下
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
        let scaled_semi_major = self.semi_major() * factor;
        let scaled_semi_minor = self.semi_minor() * factor;

        // 半軸長の幾何学的制約チェック（トレランスベース）
        let min_axis = T::DISTANCE_TOLERANCE;
        if scaled_semi_major <= min_axis || scaled_semi_minor <= min_axis {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半軸長(major:{:?}, minor:{:?})がトレランス({:?})以下になります",
                scaled_semi_major, scaled_semi_minor, min_axis
            )));
        }

        // 数値安定性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_semi_major.is_finite()
            || !scaled_semi_minor.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        Self::new(
            scaled_center,
            scaled_semi_major,
            scaled_semi_minor,
            self.rotation(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の楕円の作成に失敗しました".to_string(),
        ))
    }

    /// 半軸長スケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// この楕円に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_axis = T::DISTANCE_TOLERANCE;
        let current_major = self.semi_major();
        let current_minor = self.semi_minor();

        // 小さい方の軸を基準に計算
        let smaller_axis = if current_major < current_minor {
            current_major
        } else {
            current_minor
        };

        if smaller_axis <= T::ZERO {
            T::ZERO
        } else {
            // 最小軸長を維持するための倍率 + 安全マージン
            let min_factor = min_axis / smaller_axis;
            min_factor + T::DISTANCE_TOLERANCE
        }
    }

    /// トレランス制約付き半軸長スケール
    ///
    /// # 引数
    /// * `major_factor` - 長軸のスケール倍率
    /// * `minor_factor` - 短軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ellipse2D)` - スケール後の楕円
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    pub fn safe_scale_axes_with_tolerance(
        &self,
        major_factor: T,
        minor_factor: T,
    ) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if major_factor <= T::ZERO
            || minor_factor <= T::ZERO
            || !major_factor.is_finite()
            || !minor_factor.is_finite()
        {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        let new_semi_major = self.semi_major() * major_factor;
        let new_semi_minor = self.semi_minor() * minor_factor;

        // 半軸長の幾何学的制約チェック（トレランスベース）
        let min_axis = T::DISTANCE_TOLERANCE;
        if new_semi_major <= min_axis || new_semi_minor <= min_axis {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半軸長(major:{:?}, minor:{:?})がトレランス({:?})以下になります",
                new_semi_major, new_semi_minor, min_axis
            )));
        }

        // 楕円の条件確認（長軸 >= 短軸）
        let (final_major, final_minor) = if new_semi_major >= new_semi_minor {
            (new_semi_major, new_semi_minor)
        } else {
            (new_semi_minor, new_semi_major)
        };

        Self::new(self.center(), final_major, final_minor, self.rotation()).ok_or(
            TransformError::InvalidGeometry("軸スケール後の楕円の作成に失敗しました".to_string()),
        )
    }
}
