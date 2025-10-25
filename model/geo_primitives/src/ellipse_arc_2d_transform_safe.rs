//! EllipseArc2D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Ellipse2D, EllipseArc2D, Point2D, Vector2D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// EllipseArc2Dの安全な変換操作
impl<T: Scalar> EllipseArc2D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 移動後の楕円弧
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite() || !translation.y().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }

        // 基底楕円の安全な平行移動
        let translated_ellipse = self.ellipse().safe_translate(translation)?;

        Ok(Self::new(
            translated_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 回転後の楕円弧
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
    /// * `Ok(EllipseArc2D)` - 回転後の楕円弧
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

        // 基底楕円の安全な回転
        let rotated_ellipse = self.ellipse().safe_rotate(center, angle)?;

        // 楕円弧の角度も回転の影響を受ける（楕円の回転により相対角度が変化）
        // ただし、楕円弧自体の角度範囲は基底楕円に対して相対的なので変更なし
        Ok(Self::new(
            rotated_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - スケール後の楕円弧
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
    /// * `Ok(EllipseArc2D)` - スケール後の楕円弧
    /// * `Err(TransformError)` - 無效な入力（0以下倍率、無限大、NaN）
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

        // 基底楕円の安全なスケール
        let scaled_ellipse = self.ellipse().safe_scale(center, factor)?;

        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - スケール後の楕円弧
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
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - スケール後の楕円弧
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

        // 基底楕円の安全な非均一スケール
        let scaled_ellipse = self
            .ellipse()
            .safe_scale_non_uniform(center, scale_x, scale_y)?;

        // 非均一スケールでは楕円の軸が変化する可能性があるが、
        // 楕円弧の角度範囲は基底楕円に対して相対的なので変更なし
        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な反射（軸に対する）
    ///
    /// # 引数
    /// * `axis_point` - 反射軸上の点
    /// * `axis_direction` - 反射軸の方向ベクトル
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 反射後の楕円弧
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

        // 基底楕円の安全な反射
        let reflected_ellipse = self.ellipse().safe_reflect(axis_point, axis_direction)?;

        // 反射により楕円弧の向きが逆転するため、開始角と終了角を交換し、角度を調整
        let axis_normalized = axis_direction.normalize();
        let axis_angle = axis_normalized.y().atan2(axis_normalized.x());
        let two = T::from_f64(2.0);

        // 各角度を反射軸に対して反射（反射では角度の向きが逆転し、開始と終了が入れ替わる）
        let reflected_start = two * axis_angle - self.end_angle().to_radians();
        let reflected_end = two * axis_angle - self.start_angle().to_radians();

        // 角度を-π〜πの範囲に正規化
        let pi = T::from_f64(std::f64::consts::PI);
        let two_pi = T::from_f64(2.0 * std::f64::consts::PI);

        let mut normalized_start = reflected_start;
        while normalized_start > pi {
            normalized_start -= two_pi;
        }
        while normalized_start <= -pi {
            normalized_start += two_pi;
        }

        let mut normalized_end = reflected_end;
        while normalized_end > pi {
            normalized_end -= two_pi;
        }
        while normalized_end <= -pi {
            normalized_end += two_pi;
        }

        let reflected_start_angle = Angle::from_radians(normalized_start);
        let reflected_end_angle = Angle::from_radians(normalized_end);

        Ok(Self::new(
            reflected_ellipse,
            reflected_start_angle,
            reflected_end_angle,
        ))
    }

    /// 安全な角度範囲変更（基底楕円固定）
    ///
    /// # 引数
    /// * `new_start_angle` - 新しい開始角度
    /// * `new_end_angle` - 新しい終了角度
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 新しい角度範囲の楕円弧
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_with_angles(
        &self,
        new_start_angle: Angle<T>,
        new_end_angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        if !new_start_angle.to_radians().is_finite() || !new_end_angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        Ok(Self::new(*self.ellipse(), new_start_angle, new_end_angle))
    }

    /// 安全な基底楕円変更（角度範囲固定）
    ///
    /// # 引数
    /// * `new_ellipse` - 新しい基底楕円
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 新しい基底楕円の楕円弧
    /// * `Err(TransformError)` - 無効な楕円（作成済みなので基本的にエラーなし）
    pub fn safe_with_ellipse(&self, new_ellipse: Ellipse2D<T>) -> Result<Self, TransformError> {
        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))
    }

    /// 安全な楕円弧の部分取得（角度範囲の絞り込み）
    ///
    /// # 引数
    /// * `new_start_angle` - 新しい開始角度（現在の範囲内である必要がある）
    /// * `new_end_angle` - 新しい終了角度（現在の範囲内である必要がある）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 部分楕円弧
    /// * `Err(TransformError)` - 無効な角度または範囲外
    pub fn safe_sub_arc(
        &self,
        new_start_angle: Angle<T>,
        new_end_angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        let start_rad = new_start_angle.to_radians();
        let end_rad = new_end_angle.to_radians();
        if !start_rad.is_finite() || !end_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        // 角度範囲が現在の楕円弧の範囲内にあるかチェック
        let current_start = self.start_angle().to_radians();
        let current_end = self.end_angle().to_radians();

        // 簡単な範囲チェック（角度の正規化は省略）
        let tolerance = T::from_f64(1e-10);
        if start_rad < current_start - tolerance
            || start_rad > current_end + tolerance
            || end_rad < current_start - tolerance
            || end_rad > current_end + tolerance
        {
            return Err(TransformError::InvalidGeometry(
                "指定された角度範囲が現在の楕円弧の範囲外".to_string(),
            ));
        }

        Ok(Self::new(*self.ellipse(), new_start_angle, new_end_angle))
    }

    /// 安全な楕円弧の逆転（開始角と終了角を交換）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 逆転した楕円弧
    /// * `Err(TransformError)` - 基本的にエラーなし
    pub fn safe_reverse(&self) -> Result<Self, TransformError> {
        Ok(Self::new(
            *self.ellipse(),
            self.end_angle(),
            self.start_angle(),
        ))
    }

    /// 安全な楕円弧の長半軸スケール（角度範囲固定）
    ///
    /// # 引数
    /// * `factor` - 長半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 長半軸スケール後の楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_major(&self, factor: T) -> Result<Self, TransformError> {
        // 基底楕円の長半軸スケール
        let scaled_ellipse = self.ellipse().safe_scale_semi_major(factor)?;

        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な楕円弧の短半軸スケール（角度範囲固定）
    ///
    /// # 引数
    /// * `factor` - 短半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 短半軸スケール後の楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_minor(&self, factor: T) -> Result<Self, TransformError> {
        // 基底楕円の短半軸スケール
        let scaled_ellipse = self.ellipse().safe_scale_semi_minor(factor)?;

        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な楕円弧の回転角変更（基底楕円の回転）
    ///
    /// # 引数
    /// * `new_rotation` - 新しい回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 新しい回転角の楕円弧
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_with_ellipse_rotation(
        &self,
        new_rotation: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 基底楕円の回転角変更
        let rotated_ellipse = self.ellipse().safe_with_rotation(new_rotation)?;

        Ok(Self::new(
            rotated_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な楕円弧の中心変更
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 新しい中心の楕円弧
    /// * `Err(TransformError)` - 無効な中心点（無限大、NaN）
    pub fn safe_with_center(&self, new_center: Point2D<T>) -> Result<Self, TransformError> {
        // 基底楕円の中心変更
        let centered_ellipse = self.ellipse().safe_with_center(new_center)?;

        Ok(Self::new(
            centered_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な楕円弧の軸長変更
    ///
    /// # 引数
    /// * `new_semi_major` - 新しい長半軸
    /// * `new_semi_minor` - 新しい短半軸
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc2D)` - 新しい軸長の楕円弧
    /// * `Err(TransformError)` - 無効な軸長（0以下、無限大、NaN、長軸<短軸）
    pub fn safe_with_axes(
        &self,
        new_semi_major: T,
        new_semi_minor: T,
    ) -> Result<Self, TransformError> {
        // 基底楕円の軸長変更
        let resized_ellipse = self
            .ellipse()
            .safe_with_axes(new_semi_major, new_semi_minor)?;

        Ok(Self::new(
            resized_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ellipse2D;

    fn create_test_ellipse_arc() -> EllipseArc2D<f64> {
        let ellipse = Ellipse2D::new(
            Point2D::new(2.0, 3.0),
            4.0,                         // 長半軸
            2.0,                         // 短半軸
            std::f64::consts::FRAC_PI_6, // 30度回転
        )
        .unwrap();
        EllipseArc2D::new(
            ellipse,
            Angle::from_degrees(45.0),  // 開始角
            Angle::from_degrees(135.0), // 終了角
        )
    }

    #[test]
    fn test_safe_translate_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let translation = Vector2D::new(3.0, -2.0);
        let result = ellipse_arc.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        // 中心が移動
        assert!((result.ellipse().center().x() - 5.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 1.0).abs() < tolerance);
        // 軸長と回転角は変わらない
        assert!((result.ellipse().semi_major() - 4.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 2.0).abs() < tolerance);
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_6).abs() < tolerance);
        // 角度範囲は変わらない
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let ellipse_arc = create_test_ellipse_arc();

        // 無限大の移動ベクトル
        let invalid_translation = Vector2D::new(f64::INFINITY, 0.0);
        let result = ellipse_arc.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector2D::new(f64::NAN, 0.0);
        let result = ellipse_arc.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_origin_success() {
        let center = Point2D::new(3.0, 0.0);
        let ellipse = Ellipse2D::new(center, 2.0, 1.0, 0.0).unwrap();
        let ellipse_arc =
            EllipseArc2D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0));

        // 90度回転
        let result = ellipse_arc
            .safe_rotate_origin(Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // (3,0) が (0,3) に回転
        assert!((result.ellipse().center().x() - 0.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 3.0).abs() < tolerance);
        // 軸長は変わらない
        assert!((result.ellipse().semi_major() - 2.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 1.0).abs() < tolerance);
        // 楕円の回転角が90度増加
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_2).abs() < tolerance);
        // 弧の角度範囲は変わらない（基底楕円に対して相対的）
        assert!((result.start_angle().to_degrees() - 0.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc.safe_scale_origin(1.5).unwrap();

        let tolerance = 1e-10;
        // 中心と軸長がともに1.5倍
        assert!((result.ellipse().center().x() - 3.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 4.5).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 6.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 3.0).abs() < tolerance);
        // 回転角と弧の角度は変わらない
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_6).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc.safe_scale_origin(0.0);
        assert!(result.is_err());

        let result = ellipse_arc.safe_scale_origin(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let center = Point2D::new(1.0, 1.0);
        let ellipse = Ellipse2D::new(center, 3.0, 2.0, 0.0).unwrap(); // 軸が座標軸に平行
        let ellipse_arc = EllipseArc2D::new(
            ellipse,
            Angle::from_degrees(30.0),
            Angle::from_degrees(120.0),
        );

        let result = ellipse_arc.safe_scale_non_uniform_origin(2.0, 1.5).unwrap();

        let tolerance = 1e-10;
        // 中心がスケール
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 1.5).abs() < tolerance);

        // 軸長も変化（非均一スケールの効果）
        assert!((result.ellipse().semi_major() - 6.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 3.0).abs() < tolerance);

        // 弧の角度範囲は変わらない
        assert!((result.start_angle().to_degrees() - 30.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 120.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let center = Point2D::new(2.0, 1.0);
        let ellipse = Ellipse2D::new(center, 3.0, 2.0, 0.0).unwrap();
        let ellipse_arc =
            EllipseArc2D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0));

        // Y軸に対する反射
        let axis_point = Point2D::origin();
        let axis_direction = Vector2D::new(0.0, 1.0);
        let result = ellipse_arc
            .safe_reflect(axis_point, axis_direction)
            .unwrap();

        let tolerance = 1e-10;
        // X座標が反転される
        assert!((result.ellipse().center().x() - (-2.0)).abs() < tolerance);
        assert!((result.ellipse().center().y() - 1.0).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 3.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 2.0).abs() < tolerance);
        // Y軸に対する反射：0°→180°, 90°→90°、さらに向きが逆転
        // 元の弧0°-90°が、反射・逆転により90°-180°になる
        assert!((result.start_angle().to_degrees() - 90.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 180.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_angles_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc
            .safe_with_angles(Angle::from_degrees(60.0), Angle::from_degrees(150.0))
            .unwrap();

        let tolerance = 1e-10;
        // 基底楕円は変わらず、角度のみ変更
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 3.0).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 4.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 2.0).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 60.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 150.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_sub_arc_success() {
        let ellipse_arc = create_test_ellipse_arc(); // 45度-135度の弧
        let result = ellipse_arc
            .safe_sub_arc(
                Angle::from_degrees(60.0),  // 45度以上
                Angle::from_degrees(120.0), // 135度以下
            )
            .unwrap();

        let tolerance = 1e-10;
        // 基底楕円は変わらず、角度範囲が狭まる
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 60.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 120.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_sub_arc_out_of_range_error() {
        let ellipse_arc = create_test_ellipse_arc(); // 45度-135度の弧

        // 範囲外の開始角度
        let result = ellipse_arc.safe_sub_arc(
            Angle::from_degrees(30.0), // 45度未満
            Angle::from_degrees(120.0),
        );
        assert!(result.is_err());

        // 範囲外の終了角度
        let result = ellipse_arc.safe_sub_arc(
            Angle::from_degrees(60.0),
            Angle::from_degrees(150.0), // 135度超過
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_reverse_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc.safe_reverse().unwrap();

        let tolerance = 1e-10;
        // 基底楕円は変わらず、開始角と終了角が交換
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 135.0).abs() < tolerance); // 元の終了角
        assert!((result.end_angle().to_degrees() - 45.0).abs() < tolerance); // 元の開始角
    }

    #[test]
    fn test_safe_scale_semi_major_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc.safe_scale_semi_major(1.25).unwrap();

        let tolerance = 1e-10;
        // 中心・短軸・回転・角度は変わらず、長半軸のみ1.25倍
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 3.0).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 5.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 2.0).abs() < tolerance);
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_6).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_center_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let new_center = Point2D::new(10.0, -5.0);
        let result = ellipse_arc.safe_with_center(new_center).unwrap();

        let tolerance = 1e-10;
        // 軸長・回転・角度は変わらず、中心のみ変更
        assert!((result.ellipse().center().x() - 10.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - (-5.0)).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 4.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 2.0).abs() < tolerance);
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_6).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_axes_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let result = ellipse_arc.safe_with_axes(5.0, 3.0).unwrap();

        let tolerance = 1e-10;
        // 中心・回転・角度は変わらず、軸長のみ変更
        assert!((result.ellipse().center().x() - 2.0).abs() < tolerance);
        assert!((result.ellipse().center().y() - 3.0).abs() < tolerance);
        assert!((result.ellipse().semi_major() - 5.0).abs() < tolerance);
        assert!((result.ellipse().semi_minor() - 3.0).abs() < tolerance);
        assert!((result.ellipse().rotation() - std::f64::consts::FRAC_PI_6).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_zero_axis_error() {
        let ellipse_arc = create_test_ellipse_arc();
        let axis_point = Point2D::origin();
        let zero_axis = Vector2D::new(0.0, 0.0);
        let result = ellipse_arc.safe_reflect(axis_point, zero_axis);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let ellipse_arc = create_test_ellipse_arc();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = ellipse_arc.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
    }
}
