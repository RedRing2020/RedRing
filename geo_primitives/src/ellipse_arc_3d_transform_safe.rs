//! EllipseArc3D Safe Transform operations with Result<T, TransformError> pattern
//! EllipseArc3D Safe Transform (一時的に空実装)
//! EllipseArc3D Safe Transform 実装
//! EllipseArc3D 安全な変換エラーハンドリング実装
//! EllipseArc3D 安全な変換エラーハンドリング実装
//! EllipseArc3D Safe Transform 実装

use crate::EllipseArc3D;

use geo_foundation::Scalar;

use crate::EllipseArc3D;//!

impl<T: Scalar> EllipseArc3D<T> {

    // 将来的にSafe Transform実装を追加予定use geo_foundation::Scalar;

}
//! 安全な変換操作のエラーハンドリング//!

impl<T: Scalar> EllipseArc3D<T> {

    // 将来的にSafe Transform実装を追加予定

}
use crate::{Ellipse3D, EllipseArc3D, Point3D, Vector3D};//! Result<T, TransformError>パターンによる安全な変換操作//!//!

use geo_foundation::{Angle, Scalar, TransformError};



impl<T: Scalar> EllipseArc3D<T> {

    /// 安全な平行移動use crate::{Ellipse3D, EllipseArc3D, Point3D, Vector3D};//! Result<T, TransformError>パターンによる安全な変換操作//! Result<T, Error> を返すSafe Transform操作

    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {

        // 基本実装：通常の平行移動を呼び出しuse geo_foundation::{Angle, Scalar, TransformError};

        Ok(self.translate(translation))

    }//! エラーハンドリング付きの変換メソッド群



    /// 安全な回転（Z軸中心）/// EllipseArc3Dの安全な変換操作

    pub fn safe_rotate_z(&self, center: Point3D<T>, angle: Angle<T>) -> Result<Self, TransformError> {

        // 基本実装：通常の回転を呼び出しimpl<T: Scalar> EllipseArc3D<T> {use crate::{Ellipse3D, EllipseArc3D, Point3D, Vector3D};

        Ok(self.rotate_z(center, angle))

    }    /// 安全な平行移動



    /// 安全な均一スケール    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {use geo_foundation::{Angle, Scalar, TransformError};use crate::{EllipseArc3D, Point3D, Vector3D};

    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {

        // 基本実装：通常のスケールを呼び出し        // 移動ベクトルの有効性チェック

        Ok(self.scale(center, factor))

    }        if !translation.x().is_finite() || !translation.y().is_finite() || !translation.z().is_finite() {use geo_foundation::{Angle, Scalar};



    /// 安全な角度範囲変更            return Err(TransformError::InvalidVector);

    pub fn safe_with_angles(&self, start_angle: Angle<T>, end_angle: Angle<T>) -> Result<Self, TransformError> {

        Ok(Self::new(self.ellipse().clone(), start_angle, end_angle))        }/// EllipseArc3Dの安全な変換操作

    }



    /// 安全な基底楕円変更

    pub fn safe_with_ellipse(&self, new_ellipse: Ellipse3D<T>) -> Result<Self, TransformError> {        let translated_ellipse = match self.ellipse().safe_translate(translation) {impl<T: Scalar> EllipseArc3D<T> {// ============================================================================

        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))

    }            Ok(ellipse) => ellipse,



    /// 安全な部分弧抽出            Err(_) => return Err(TransformError::InvalidVector),    /// 安全な平行移動// Safe Transform Error Types

    pub fn safe_sub_arc(&self, sub_start: Angle<T>, sub_end: Angle<T>) -> Result<Self, TransformError> {

        match self.sub_arc(sub_start, sub_end) {        };

            Some(arc) => Ok(arc),

            None => Err(TransformError::InvalidAngle),    ///// ============================================================================

        }

    }        Ok(Self::new(translated_ellipse, self.start_angle(), self.end_angle()))

}

    }    /// # 引数

#[cfg(test)]

mod tests {

    use super::*;

    use crate::{Point3D, Vector3D};    /// 安全な回転（Z軸中心）    /// * `translation` - 移動ベクトル#[derive(Debug, Clone, Copy, PartialEq, Eq)]



    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {    pub fn safe_rotate_z(&self, center: Point3D<T>, angle: Angle<T>) -> Result<Self, TransformError> {

        let center = Point3D::new(0.0, 0.0, 0.0);

        let normal = Vector3D::new(0.0, 0.0, 1.0);        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {    ///pub enum TransformError {

        let major_axis = Vector3D::new(1.0, 0.0, 0.0);

        let ellipse = Ellipse3D::new(center, 2.0, 1.0, normal, major_axis).unwrap();            return Err(TransformError::InvalidPoint);

        EllipseArc3D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0))

    }        }    /// # 戻り値    InvalidVector,



    #[test]

    fn test_safe_translate_success() {

        let ellipse_arc = create_test_ellipse_arc();        if !angle.to_radians().is_finite() {    /// * `Ok(EllipseArc3D)` - 移動後の楕円弧    VectorTooLarge,

        let translation = Vector3D::new(1.0, 2.0, 3.0);

                    return Err(TransformError::InvalidAngle);

        let result = ellipse_arc.safe_translate(translation);

        assert!(result.is_ok());        }    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）    InvalidPoint,

    }



    #[test]

    fn test_safe_scale_success() {        let rotated_ellipse = match self.ellipse().safe_rotate(center, Vector3D::unit_z(), angle) {    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {    InvalidAngle,

        let ellipse_arc = create_test_ellipse_arc();

        let center = Point3D::origin();            Ok(ellipse) => ellipse,

        let factor = 2.0;

                    Err(_) => return Err(TransformError::InvalidPoint),        // 移動ベクトルの有効性チェック    AngleTooLarge,

        let result = ellipse_arc.safe_scale(center, factor);

        assert!(result.is_ok());        };

    }

        if !translation.x().is_finite() || !translation.y().is_finite() || !translation.z().is_finite() {    ZeroVector,

    #[test]

    fn test_safe_with_angles_success() {        Ok(Self::new(rotated_ellipse, self.start_angle(), self.end_angle()))

        let ellipse_arc = create_test_ellipse_arc();

        let new_start = Angle::from_degrees(45.0);    }            return Err(TransformError::InvalidVector);    InvalidScale,

        let new_end = Angle::from_degrees(135.0);



        let result = ellipse_arc.safe_with_angles(new_start, new_end);

        assert!(result.is_ok());    /// 安全な均一スケール        }    ScaleTooExtreme,

    }

    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {

    #[test]

    fn test_safe_sub_arc_success() {        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {    InvalidResult,

        let ellipse_arc = create_test_ellipse_arc();

        let sub_start = Angle::from_degrees(30.0);            return Err(TransformError::InvalidPoint);

        let sub_end = Angle::from_degrees(60.0);

                }        // 基底楕円を安全に平行移動}

        let result = ellipse_arc.safe_sub_arc(sub_start, sub_end);

        assert!(result.is_ok());

    }

}        if !factor.is_finite() || factor <= T::ZERO {        let translated_ellipse = match self.ellipse().safe_translate(translation) {

            return Err(TransformError::ZeroScale);

        }            Ok(ellipse) => ellipse,impl<T: Scalar> EllipseArc3D<T> {



        let scaled_ellipse = match self.ellipse().safe_scale(center, factor) {            Err(_) => return Err(TransformError::InvalidVector),    /// 安全な平行移動

            Ok(ellipse) => ellipse,

            Err(_) => return Err(TransformError::ZeroScale),        };    ///

        };

    /// ベクトルの妥当性を検証してから平行移動を実行

        Ok(Self::new(scaled_ellipse, self.start_angle(), self.end_angle()))

    }        Ok(Self::new(translated_ellipse, self.start_angle(), self.end_angle()))    ///



    /// 安全な非等方スケール    }    /// # 引数

    pub fn safe_scale_non_uniform(

        &self,    /// * `translation` - 移動ベクトル

        center: Point3D<T>,

        scale_x: T,    /// 安全な回転（Z軸中心）    ///

        scale_y: T,

        scale_z: T,    ///    /// # 戻り値

    ) -> Result<Self, TransformError> {

        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {    /// # 引数    /// 成功時は平行移動された楕円弧、失敗時はTransformError

            return Err(TransformError::InvalidPoint);

        }    /// * `center` - 回転中心点    ///



        if !scale_x.is_finite() || scale_x <= T::ZERO ||    /// * `angle` - 回転角度（Angle型）    /// # エラー

           !scale_y.is_finite() || scale_y <= T::ZERO ||

           !scale_z.is_finite() || scale_z <= T::ZERO {    ///    /// * `InvalidVector` - 移動ベクトルが無効（NaN、無限大を含む）

            return Err(TransformError::ZeroScale);

        }    /// # 戻り値    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {



        let scaled_ellipse = match self.ellipse().safe_scale_non_uniform(center, scale_x, scale_y, scale_z) {    /// * `Ok(EllipseArc3D)` - 回転後の楕円弧        // ベクトルの妥当性チェック

            Ok(ellipse) => ellipse,

            Err(_) => return Err(TransformError::ZeroScale),    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）        if !translation.is_finite() {

        };

    pub fn safe_rotate_z(&self, center: Point3D<T>, angle: Angle<T>) -> Result<Self, TransformError> {            return Err(TransformError::InvalidVector);

        Ok(Self::new(scaled_ellipse, self.start_angle(), self.end_angle()))

    }        // 回転中心点の有効性チェック        }



    /// 安全な角度範囲変更        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {

    pub fn safe_with_angles(&self, start_angle: Angle<T>, end_angle: Angle<T>) -> Result<Self, TransformError> {

        if !start_angle.to_radians().is_finite() || !end_angle.to_radians().is_finite() {            return Err(TransformError::InvalidPoint);        if translation.magnitude() > T::from_f64(1e12) {

            return Err(TransformError::InvalidAngle);

        }        }            return Err(TransformError::VectorTooLarge);



        Ok(Self::new(self.ellipse().clone(), start_angle, end_angle))        }

    }

        // 回転角度の有効性チェック

    /// 安全な基底楕円変更

    pub fn safe_with_ellipse(&self, new_ellipse: Ellipse3D<T>) -> Result<Self, TransformError> {        if !angle.to_radians().is_finite() {        Ok(self.translate(translation))

        if new_ellipse.semi_major_axis() <= T::ZERO || new_ellipse.semi_minor_axis() <= T::ZERO {

            return Err(TransformError::ZeroScale);            return Err(TransformError::InvalidAngle);    }

        }

        }

        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))

    }    /// 安全なスケール



    /// 安全な部分弧抽出        // 基底楕円を安全に回転（Z軸は3つの引数を取る）    ///

    pub fn safe_sub_arc(&self, sub_start: Angle<T>, sub_end: Angle<T>) -> Result<Self, TransformError> {

        if !sub_start.to_radians().is_finite() || !sub_end.to_radians().is_finite() {        let rotated_ellipse = match self.ellipse().safe_rotate(center, Vector3D::unit_z(), angle) {    /// スケール倍率の妥当性を検証してからスケールを実行

            return Err(TransformError::InvalidAngle);

        }            Ok(ellipse) => ellipse,    ///



        let start_rad = self.start_angle().to_radians();            Err(_) => return Err(TransformError::InvalidPoint),    /// # 引数

        let end_rad = self.end_angle().to_radians();

        let sub_start_rad = sub_start.to_radians();        };    /// * `factor` - スケール倍率

        let sub_end_rad = sub_end.to_radians();

    ///

        if sub_start_rad < start_rad || sub_start_rad > end_rad ||

           sub_end_rad < start_rad || sub_end_rad > end_rad ||         Ok(Self::new(rotated_ellipse, self.start_angle(), self.end_angle()))    /// # 戻り値

           sub_start_rad > sub_end_rad {

            return Err(TransformError::InvalidAngle);    }    /// 成功時はスケールされた楕円弧、失敗時はTransformError

        }

    ///

        Ok(Self::new(self.ellipse().clone(), sub_start, sub_end))

    }    /// 安全な均一スケール（指定中心点）    /// # エラー

}

    ///    /// * `InvalidScale` - スケール倍率が無効（NaN、無限大、ゼロ、負）

#[cfg(test)]

mod tests {    /// # 引数    pub fn safe_scale(&self, factor: T) -> Result<Self, TransformError> {

    use super::*;

    use crate::{Point3D, Vector3D};    /// * `center` - スケール中心点        // スケール倍率の妥当性チェック



    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {    /// * `factor` - スケール倍率        if !factor.is_finite() {

        let center = Point3D::new(0.0, 0.0, 0.0);

        let normal = Vector3D::new(0.0, 0.0, 1.0);    ///            return Err(TransformError::InvalidScale);

        let major_axis = Vector3D::new(1.0, 0.0, 0.0);

        let ellipse = Ellipse3D::new(center, 2.0, 1.0, normal, major_axis).unwrap();    /// # 戻り値        }

        EllipseArc3D::new(ellipse, Angle::from_degrees(0.0), Angle::from_degrees(90.0))

    }    /// * `Ok(EllipseArc3D)` - スケール後の楕円弧



    #[test]    /// * `Err(TransformError)` - 無効な入力（ゼロ、負数、無限大、NaN）        if factor <= T::ZERO {

    fn test_safe_translate_success() {

        let ellipse_arc = create_test_ellipse_arc();    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {            return Err(TransformError::InvalidScale);

        let translation = Vector3D::new(1.0, 2.0, 3.0);

                // スケール中心点の有効性チェック        }

        let result = ellipse_arc.safe_translate(translation);

        assert!(result.is_ok());        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {



        let translated = result.unwrap();            return Err(TransformError::InvalidPoint);        if factor < T::from_f64(1e-12) || factor > T::from_f64(1e12) {

        assert_eq!(translated.center().x(), 1.0);

        assert_eq!(translated.center().y(), 2.0);        }            return Err(TransformError::ScaleTooExtreme);

        assert_eq!(translated.center().z(), 3.0);

    }        }



    #[test]        // スケール倍率の有効性チェック

    fn test_safe_translate_invalid_vector() {

        let ellipse_arc = create_test_ellipse_arc();        if !factor.is_finite() || factor <= T::ZERO {        Ok(self.translate(Vector3D::zero())) // TODO: 実装待ち

        let invalid_translation = Vector3D::new(f64::INFINITY, 2.0, 3.0);

                    return Err(TransformError::ZeroScale);    }

        let result = ellipse_arc.safe_translate(invalid_translation);

        assert!(result.is_err());        }

        assert!(matches!(result.unwrap_err(), TransformError::InvalidVector));

    }    /// 安全な非等方スケール



    #[test]        // 基底楕円を安全にスケール    ///

    fn test_safe_scale_success() {

        let ellipse_arc = create_test_ellipse_arc();        let scaled_ellipse = match self.ellipse().safe_scale(center, factor) {    /// 各軸のスケール倍率の妥当性を検証してから非等方スケールを実行

        let center = Point3D::origin();

        let factor = 2.0;            Ok(ellipse) => ellipse,    ///



        let result = ellipse_arc.safe_scale(center, factor);            Err(_) => return Err(TransformError::ZeroScale),    /// # 引数

        assert!(result.is_ok());

                };    /// * `scale_x` - X軸方向のスケール倍率

        let scaled = result.unwrap();

        assert_eq!(scaled.semi_major(), 4.0);    /// * `scale_y` - Y軸方向のスケール倍率

        assert_eq!(scaled.semi_minor(), 2.0);

    }        Ok(Self::new(scaled_ellipse, self.start_angle(), self.end_angle()))    /// * `scale_z` - Z軸方向のスケール倍率



    #[test]    }    ///

    fn test_safe_scale_zero_factor_error() {

        let ellipse_arc = create_test_ellipse_arc();    /// # 戻り値

        let center = Point3D::origin();

        let zero_factor = 0.0;    /// 安全な非等方スケール    /// 成功時は非等方スケールされた楕円弧、失敗時はTransformError



        let result = ellipse_arc.safe_scale(center, zero_factor);    ///    ///

        assert!(result.is_err());

        assert!(matches!(result.unwrap_err(), TransformError::ZeroScale));    /// # 引数    /// # エラー

    }

    /// * `center` - スケール中心点    /// * `InvalidScale` - いずれかのスケール倍率が無効

    #[test]

    fn test_safe_with_angles_success() {    /// * `scale_x` - X軸方向のスケール倍率    pub fn safe_scale_non_uniform(

        let ellipse_arc = create_test_ellipse_arc();

        let new_start = Angle::from_degrees(45.0);    /// * `scale_y` - Y軸方向のスケール倍率        &self,

        let new_end = Angle::from_degrees(135.0);

            /// * `scale_z` - Z軸方向のスケール倍率        scale_x: T,

        let result = ellipse_arc.safe_with_angles(new_start, new_end);

        assert!(result.is_ok());    ///        scale_y: T,



        let modified = result.unwrap();    /// # 戻り値        scale_z: T,

        assert_eq!(modified.start_angle().to_degrees(), 45.0);

        assert_eq!(modified.end_angle().to_degrees(), 135.0);    /// * `Ok(EllipseArc3D)` - 非等方スケール後の楕円弧    ) -> Result<Self, TransformError> {

    }

    /// * `Err(TransformError)` - 無効な入力        // 各軸のスケール倍率チェック

    #[test]

    fn test_safe_sub_arc_success() {    pub fn safe_scale_non_uniform(        let factors = [scale_x, scale_y, scale_z];

        let ellipse_arc = create_test_ellipse_arc();

        let sub_start = Angle::from_degrees(30.0);        &self,        for &factor in &factors {

        let sub_end = Angle::from_degrees(60.0);

                center: Point3D<T>,            if !factor.is_finite() {

        let result = ellipse_arc.safe_sub_arc(sub_start, sub_end);

        assert!(result.is_ok());        scale_x: T,                return Err(TransformError::InvalidScale);



        let sub_arc = result.unwrap();        scale_y: T,            }

        assert_eq!(sub_arc.start_angle().to_degrees(), 30.0);

        assert_eq!(sub_arc.end_angle().to_degrees(), 60.0);        scale_z: T,

    }

    ) -> Result<Self, TransformError> {            if factor <= T::ZERO {

    #[test]

    fn test_safe_sub_arc_out_of_range_error() {        // スケール中心点の有効性チェック                return Err(TransformError::InvalidScale);

        let ellipse_arc = create_test_ellipse_arc();

        let invalid_start = Angle::from_degrees(-30.0);        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {            }

        let invalid_end = Angle::from_degrees(60.0);

                    return Err(TransformError::InvalidPoint);

        let result = ellipse_arc.safe_sub_arc(invalid_start, invalid_end);

        assert!(result.is_err());        }            if factor < T::from_f64(1e-12) || factor > T::from_f64(1e12) {

        assert!(matches!(result.unwrap_err(), TransformError::InvalidAngle));

    }                return Err(TransformError::ScaleTooExtreme);

}
        // スケール倍率の有効性チェック            }

        if !scale_x.is_finite() || scale_x <= T::ZERO {        }

            return Err(TransformError::ZeroScale);

        }        Ok(self.scale_non_uniform(Point3D::origin(), scale_x, scale_y, scale_z))

        if !scale_y.is_finite() || scale_y <= T::ZERO {    }

            return Err(TransformError::ZeroScale);}

        }

        if !scale_z.is_finite() || scale_z <= T::ZERO {// ============================================================================

            return Err(TransformError::ZeroScale);// Safe Composite Transform Operations

        }// ============================================================================



        // 基底楕円を非等方スケールimpl<T: Scalar> EllipseArc3D<T> {

        let scaled_ellipse = match self.ellipse().safe_scale_non_uniform(center, scale_x, scale_y, scale_z) {    /// 安全な複合変換：平行移動 + スケール

            Ok(ellipse) => ellipse,    ///

            Err(_) => return Err(TransformError::ZeroScale),    /// # 引数

        };    /// * `translation` - 移動ベクトル

    /// * `scale_factor` - スケール倍率

        Ok(Self::new(scaled_ellipse, self.start_angle(), self.end_angle()))    ///

    }    /// # 戻り値

    /// 成功時は変換された楕円弧、失敗時は最初に発生したTransformError

    /// 安全な角度範囲変更    pub fn safe_translate_and_scale(

    ///        &self,

    /// # 引数        translation: Vector3D<T>,

    /// * `start_angle` - 新しい開始角度        scale_factor: T,

    /// * `end_angle` - 新しい終了角度    ) -> Result<Self, TransformError> {

    ///        let translated = self.safe_translate(translation)?;

    /// # 戻り値        translated.safe_scale(scale_factor)

    /// * `Ok(EllipseArc3D)` - 角度範囲変更後の楕円弧    }

    /// * `Err(TransformError)` - 無効な角度}

    pub fn safe_with_angles(&self, start_angle: Angle<T>, end_angle: Angle<T>) -> Result<Self, TransformError> {

        // 角度の有効性チェック// ============================================================================

        if !start_angle.to_radians().is_finite() || !end_angle.to_radians().is_finite() {// Validation and Error Recovery

            return Err(TransformError::InvalidAngle);// ============================================================================

        }

impl<T: Scalar> EllipseArc3D<T> {

        Ok(Self::new(self.ellipse().clone(), start_angle, end_angle))    /// 変換後の状態検証

    }    ///

    /// 変換後の楕円弧が幾何学的に有効かチェック

    /// 安全な基底楕円変更    ///

    ///    /// # 戻り値

    /// # 引数    /// 有効な場合は元の楕円弧、無効な場合はTransformError

    /// * `new_ellipse` - 新しい基底楕円    pub fn validate_after_transform(&self) -> Result<Self, TransformError> {

    ///        if !self.is_valid() {

    /// # 戻り値            return Err(TransformError::InvalidResult);

    /// * `Ok(EllipseArc3D)` - 基底楕円変更後の楕円弧        }

    /// * `Err(TransformError)` - 無効な楕円

    pub fn safe_with_ellipse(&self, new_ellipse: Ellipse3D<T>) -> Result<Self, TransformError> {        Ok(*self)

        // 楕円の有効性チェック（簡易版）    }

        if new_ellipse.semi_major_axis() <= T::ZERO || new_ellipse.semi_minor_axis() <= T::ZERO {

            return Err(TransformError::ZeroScale);    /// スケールエラーの回復

        }    ///

    /// 無効なスケール値を有効範囲にクランプして再試行

        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))    ///

    }    /// # 引数

    /// * `factor` - 元のスケール倍率

    /// 安全な部分弧抽出    ///

    ///    /// # 戻り値

    /// # 引数    /// 調整されたスケール倍率での変換結果

    /// * `sub_start` - 部分弧の開始角度    pub fn recover_scale_error(&self, factor: T) -> Self {

    /// * `sub_end` - 部分弧の終了角度        let clamped_factor = if factor <= T::ZERO {

    ///            T::from_f64(1e-6) // 最小値

    /// # 戻り値        } else if factor > T::from_f64(1e12) {

    /// * `Ok(EllipseArc3D)` - 部分弧            T::from_f64(1e12) // 最大値

    /// * `Err(TransformError)` - 角度が範囲外        } else if factor < T::from_f64(1e-12) {

    pub fn safe_sub_arc(&self, sub_start: Angle<T>, sub_end: Angle<T>) -> Result<Self, TransformError> {            T::from_f64(1e-6) // 実用的な最小値

        // 角度の有効性チェック        } else {

        if !sub_start.to_radians().is_finite() || !sub_end.to_radians().is_finite() {            factor

            return Err(TransformError::InvalidAngle);        };

        }

        self.safe_scale(clamped_factor).unwrap_or(*self)

        // 範囲チェック    }

        let start_rad = self.start_angle().to_radians();

        let end_rad = self.end_angle().to_radians();    /// ベクトルエラーの回復

        let sub_start_rad = sub_start.to_radians();    ///

        let sub_end_rad = sub_end.to_radians();    /// 無効なベクトルを正規化または代替値で置換して再試行

    ///

        if sub_start_rad < start_rad || sub_start_rad > end_rad ||     /// # 引数

           sub_end_rad < start_rad || sub_end_rad > end_rad ||     /// * `translation` - 元の移動ベクトル

           sub_start_rad > sub_end_rad {    ///

            return Err(TransformError::InvalidAngle);    /// # 戻り値

        }    /// 修正されたベクトルでの変換結果

    pub fn recover_vector_error(&self, translation: Vector3D<T>) -> Self {

        Ok(Self::new(self.ellipse().clone(), sub_start, sub_end))        let corrected_vector = if !translation.is_finite() {

    }            // NaN や無限大の場合はゼロベクトル

}            Vector3D::zero()

        } else if translation.magnitude() > T::from_f64(1e12) {

// ============================================================================            // 大きすぎる場合は正規化して適切なスケールに

// Tests            let direction = translation.normalize();

// ============================================================================            direction * T::from_f64(1000.0) // 適切なスケール

        } else {

#[cfg(test)]            translation

mod tests {        };

    use super::*;

    use crate::{Point3D, Vector3D};        self.safe_translate(corrected_vector).unwrap_or(*self)

    }

    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {}

        let center = Point3D::new(0.0, 0.0, 0.0);

        let normal = Vector3D::new(0.0, 0.0, 1.0);/// EllipseArc3Dの安全な変換操作

        let major_axis = Vector3D::new(1.0, 0.0, 0.0);impl<T: Scalar> EllipseArc3D<T> {

        let ellipse = Ellipse3D::new(center, 2.0, 1.0, normal, major_axis).unwrap();    /// 安全な平行移動

    ///

        EllipseArc3D::new(    /// # 引数

            ellipse,    /// * `translation` - 移動ベクトル

            Angle::from_degrees(0.0),    ///

            Angle::from_degrees(90.0),    /// # 戻り値

        )    /// * `Ok(EllipseArc3D)` - 移動後の3D楕円弧

    }    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）

    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {

    #[test]        // 移動ベクトルの有効性チェック

    fn test_safe_translate_success() {        if !translation.x().is_finite()

        let ellipse_arc = create_test_ellipse_arc();            || !translation.y().is_finite()

        let translation = Vector3D::new(1.0, 2.0, 3.0);            || !translation.z().is_finite()

                {

        let result = ellipse_arc.safe_translate(translation);            return Err(TransformError::InvalidInput(

        assert!(result.is_ok());                "Translation vector contains non-finite values".to_string(),

                    ));

        let translated = result.unwrap();        }

        assert_eq!(translated.center().x(), 1.0);

        assert_eq!(translated.center().y(), 2.0);        // 基底楕円の安全な平行移動

        assert_eq!(translated.center().z(), 3.0);        let translated_ellipse = self.ellipse().safe_translate(translation)?;

    }

        Ok(Self::new(

    #[test]            translated_ellipse,

    fn test_safe_translate_invalid_vector() {            self.start_angle(),

        let ellipse_arc = create_test_ellipse_arc();            self.end_angle(),

        let invalid_translation = Vector3D::new(f64::INFINITY, 2.0, 3.0);        ))

            }

        let result = ellipse_arc.safe_translate(invalid_translation);

        assert!(result.is_err());    /// 安全なZ軸回転（原点中心）

        assert!(matches!(result.unwrap_err(), TransformError::InvalidVector));    ///

    }    /// # 引数

    /// * `angle` - 回転角度（Angle型）

    #[test]    ///

    fn test_safe_scale_success() {    /// # 戻り値

        let ellipse_arc = create_test_ellipse_arc();    /// * `Ok(EllipseArc3D)` - 回転後の3D楕円弧

        let center = Point3D::origin();    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）

        let factor = 2.0;    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {

                self.safe_rotate_z(Point3D::origin(), angle)

        let result = ellipse_arc.safe_scale(center, factor);    }

        assert!(result.is_ok());

            /// 安全なZ軸回転（指定点中心）

        let scaled = result.unwrap();    ///

        assert_eq!(scaled.semi_major(), 4.0); // 2.0 * 2.0    /// # 引数

        assert_eq!(scaled.semi_minor(), 2.0); // 1.0 * 2.0    /// * `center` - 回転中心点

    }    /// * `angle` - 回転角度（Angle型）

    ///

    #[test]    /// # 戻り値

    fn test_safe_scale_zero_factor_error() {    /// * `Ok(EllipseArc3D)` - 回転後の3D楕円弧

        let ellipse_arc = create_test_ellipse_arc();    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）

        let center = Point3D::origin();    pub fn safe_rotate_z(

        let zero_factor = 0.0;        &self,

                center: Point3D<T>,

        let result = ellipse_arc.safe_scale(center, zero_factor);        angle: Angle<T>,

        assert!(result.is_err());    ) -> Result<Self, TransformError> {

        assert!(matches!(result.unwrap_err(), TransformError::ZeroScale));        // 回転中心の有効性チェック

    }        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {

            return Err(TransformError::InvalidInput(

    #[test]                "Rotation center contains non-finite values".to_string(),

    fn test_safe_with_angles_success() {            ));

        let ellipse_arc = create_test_ellipse_arc();        }

        let new_start = Angle::from_degrees(45.0);

        let new_end = Angle::from_degrees(135.0);        // 角度の有効性チェック

                let angle_rad = angle.to_radians();

        let result = ellipse_arc.safe_with_angles(new_start, new_end);        if !angle_rad.is_finite() {

        assert!(result.is_ok());            return Err(TransformError::InvalidInput(

                        "Invalid rotation angle".to_string(),

        let modified = result.unwrap();            ));

        assert_eq!(modified.start_angle().to_degrees(), 45.0);        }

        assert_eq!(modified.end_angle().to_degrees(), 135.0);

    }        // 基底楕円の安全なZ軸回転

        let rotated_ellipse = self.ellipse().safe_rotate_z(center, angle)?;

    #[test]

    fn test_safe_sub_arc_success() {        Ok(Self::new(

        let ellipse_arc = create_test_ellipse_arc(); // 0-90度            rotated_ellipse,

        let sub_start = Angle::from_degrees(30.0);            self.start_angle(),

        let sub_end = Angle::from_degrees(60.0);            self.end_angle(),

                ))

        let result = ellipse_arc.safe_sub_arc(sub_start, sub_end);    }

        assert!(result.is_ok());

            /// 安全な任意軸回転

        let sub_arc = result.unwrap();    ///

        assert_eq!(sub_arc.start_angle().to_degrees(), 30.0);    /// # 引数

        assert_eq!(sub_arc.end_angle().to_degrees(), 60.0);    /// * `axis_point` - 回転軸上の点

    }    /// * `axis_direction` - 回転軸の方向ベクトル

    /// * `angle` - 回転角度（Angle型）

    #[test]    ///

    fn test_safe_sub_arc_out_of_range_error() {    /// # 戻り値

        let ellipse_arc = create_test_ellipse_arc(); // 0-90度    /// * `Ok(EllipseArc3D)` - 回転後の3D楕円弧

        let invalid_start = Angle::from_degrees(-30.0); // 範囲外    /// * `Err(TransformError)` - 無効な入力（ゼロベクトル軸、無限大、NaN）

        let invalid_end = Angle::from_degrees(60.0);    pub fn safe_rotate_axis(

                &self,

        let result = ellipse_arc.safe_sub_arc(invalid_start, invalid_end);        axis_point: Point3D<T>,

        assert!(result.is_err());        axis_direction: Vector3D<T>,

        assert!(matches!(result.unwrap_err(), TransformError::InvalidAngle));        angle: Angle<T>,

    }    ) -> Result<Self, TransformError> {

}        // 軸点の有効性チェック
        if !axis_point.x().is_finite() || !axis_point.y().is_finite() || !axis_point.z().is_finite()
        {
            return Err(TransformError::InvalidInput(
                "Axis point contains non-finite values".to_string(),
            ));
        }

        // 軸方向ベクトルの有効性チェック
        if !axis_direction.x().is_finite()
            || !axis_direction.y().is_finite()
            || !axis_direction.z().is_finite()
        {
            return Err(TransformError::InvalidInput(
                "Axis direction contains non-finite values".to_string(),
            ));
        }

        // ゼロベクトルチェック
        if axis_direction.magnitude() <= T::EPSILON {
            return Err(TransformError::DegenerateGeometry(
                "Axis direction is zero vector".to_string(),
            ));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidInput(
                "Invalid rotation angle".to_string(),
            ));
        }

        // 基底楕円の安全な軸回転
        let rotated_ellipse =
            self.ellipse()
                .safe_rotate_around_axis(axis_point, axis_direction, angle)?;

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
    /// * `Ok(EllipseArc3D)` - スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
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
    /// * `Ok(EllipseArc3D)` - スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効な入力（0以下倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale center contains non-finite values".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale factor must be positive and finite".to_string(),
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
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
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
    /// * `Ok(EllipseArc3D)` - スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効な入力（0以下倍率、無限大、NaN）
    pub fn safe_scale_non_uniform(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale center contains non-finite values".to_string(),
            ));
        }

        // スケール倍率の有効性チェック
        if scale_x <= T::ZERO || !scale_x.is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale factor X must be positive and finite".to_string(),
            ));
        }
        if scale_y <= T::ZERO || !scale_y.is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale factor Y must be positive and finite".to_string(),
            ));
        }
        if scale_z <= T::ZERO || !scale_z.is_finite() {
            return Err(TransformError::InvalidInput(
                "Scale factor Z must be positive and finite".to_string(),
            ));
        }

        // 基底楕円の安全な非均一スケール
        let scaled_ellipse = self
            .ellipse()
            .safe_scale_non_uniform(center, scale_x, scale_y, scale_z)?;

        // 非等方スケールでは角度が変化する可能性があるため再計算
        let start_point = self.start_point();
        let end_point = self.end_point();

        let scaled_start = Point3D::new(
            center.x() + (start_point.x() - center.x()) * scale_x,
            center.y() + (start_point.y() - center.y()) * scale_y,
            center.z() + (start_point.z() - center.z()) * scale_z,
        );
        let scaled_end = Point3D::new(
            center.x() + (end_point.x() - center.x()) * scale_x,
            center.y() + (end_point.y() - center.y()) * scale_y,
            center.z() + (end_point.z() - center.z()) * scale_z,
        );

        // 角度の再計算は単純化（詳細版はtransform.rsで実装）
        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な角度範囲変更（基底楕円固定）
    ///
    /// # 引数
    /// * `new_start_angle` - 新しい開始角度
    /// * `new_end_angle` - 新しい終了角度
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 新しい角度範囲の3D楕円弧
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_with_angles(
        &self,
        new_start_angle: Angle<T>,
        new_end_angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        if !new_start_angle.to_radians().is_finite() || !new_end_angle.to_radians().is_finite() {
            return Err(TransformError::InvalidInput("Invalid angles".to_string()));
        }

        Ok(Self::new(
            self.ellipse().clone(),
            new_start_angle,
            new_end_angle,
        ))
    }

    /// 安全な基底楕円変更（角度範囲固定）
    ///
    /// # 引数
    /// * `new_ellipse` - 新しい基底楕円
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 新しい基底楕円の3D楕円弧
    /// * `Err(TransformError)` - 無効な楕円（作成済みなので基本的にエラーなし）
    pub fn safe_with_ellipse(&self, new_ellipse: Ellipse3D<T>) -> Result<Self, TransformError> {
        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))
    }

    /// 安全な3D楕円弧の部分取得（角度範囲の絞り込み）
    ///
    /// # 引数
    /// * `new_start_angle` - 新しい開始角度（現在の範囲内である必要がある）
    /// * `new_end_angle` - 新しい終了角度（現在の範囲内である必要がある）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 部分3D楕円弧
    /// * `Err(TransformError)` - 無効な角度または範囲外
    pub fn safe_sub_arc(
        &self,
        new_start_angle: Angle<T>,
        new_end_angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        if !new_start_angle.to_radians().is_finite() || !new_end_angle.to_radians().is_finite() {
            return Err(TransformError::InvalidInput("Invalid angles".to_string()));
        }

        // 角度範囲チェック
        let start_rad = self.start_angle().to_radians();
        let end_rad = self.end_angle().to_radians();
        let new_start_rad = new_start_angle.to_radians();
        let new_end_rad = new_end_angle.to_radians();

        if new_start_rad < start_rad
            || new_start_rad > end_rad
            || new_end_rad < start_rad
            || new_end_rad > end_rad
            || new_start_rad > new_end_rad
        {
            return Err(TransformError::InvalidInput(
                "Sub-arc angles are outside the original arc range".to_string(),
            ));
        }

        Ok(Self::new(
            self.ellipse().clone(),
            new_start_angle,
            new_end_angle,
        ))
    }

    /// 安全な3D楕円弧の逆転（開始角と終了角を交換）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 逆転した3D楕円弧
    /// * `Err(TransformError)` - 基本的にエラーなし
    pub fn safe_reverse(&self) -> Result<Self, TransformError> {
        Ok(Self::new(
            self.ellipse().clone(),
            self.end_angle(),
            self.start_angle(),
        ))
    }

    /// 安全な3D楕円弧の長半軸スケール（角度範囲固定）
    ///
    /// # 引数
    /// * `factor` - 長半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 長半軸スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_major(&self, factor: T) -> Result<Self, TransformError> {
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidInput(
                "Semi-major scale factor must be positive and finite".to_string(),
            ));
        }

        let scaled_ellipse = self.ellipse().safe_scale_semi_major(factor)?;
        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な3D楕円弧の短半軸スケール（角度範囲固定）
    ///
    /// # 引数
    /// * `factor` - 短半軸のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 短半軸スケール後の3D楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_semi_minor(&self, factor: T) -> Result<Self, TransformError> {
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidInput(
                "Semi-minor scale factor must be positive and finite".to_string(),
            ));
        }

        let scaled_ellipse = self.ellipse().safe_scale_semi_minor(factor)?;
        Ok(Self::new(
            scaled_ellipse,
            self.start_angle(),
            self.end_angle(),
        ))
    }

    /// 安全な3D楕円弧の中心変更
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 新しい中心の3D楕円弧
    /// * `Err(TransformError)` - 無効な中心点（無限大、NaN）
    pub fn safe_with_center(&self, new_center: Point3D<T>) -> Result<Self, TransformError> {
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(TransformError::InvalidInput(
                "Center point contains non-finite values".to_string(),
            ));
        }

        let new_ellipse = self.ellipse().safe_with_center(new_center)?;
        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))
    }

    /// 安全な3D楕円弧の軸長変更
    ///
    /// # 引数
    /// * `new_semi_major` - 新しい長半軸
    /// * `new_semi_minor` - 新しい短半軸
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 新しい軸長の3D楕円弧
    /// * `Err(TransformError)` - 無効な軸長（0以下、無限大、NaN、長軸<短軸）
    pub fn safe_with_axes(
        &self,
        new_semi_major: T,
        new_semi_minor: T,
    ) -> Result<Self, TransformError> {
        let new_ellipse = self
            .ellipse()
            .safe_with_axes(new_semi_major, new_semi_minor)?;
        Ok(Self::new(new_ellipse, self.start_angle(), self.end_angle()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ellipse3D;

    fn create_test_ellipse_arc() -> EllipseArc3D<f64> {
        let center = Point3D::new(2.0, 3.0, 1.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let major_axis = Vector3D::new(1.0, 0.0, 0.0);
        let ellipse = Ellipse3D::new(center, 4.0, 2.0, normal, major_axis).unwrap();

        EllipseArc3D::new(
            ellipse,
            Angle::from_degrees(45.0),
            Angle::from_degrees(135.0),
        )
    }

    #[test]
    fn test_safe_translate_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let translation = Vector3D::new(1.0, -1.0, 2.0);

        let result = ellipse_arc.safe_translate(translation);
        assert!(result.is_ok());

        let translated = result.unwrap();
        let tolerance = 1e-10;
        assert!((translated.center().x() - 3.0).abs() < tolerance);
        assert!((translated.center().y() - 2.0).abs() < tolerance);
        assert!((translated.center().z() - 3.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let ellipse_arc = create_test_ellipse_arc();
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);

        let result = ellipse_arc.safe_translate(invalid_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let angle = Angle::from_degrees(90.0);

        let result = ellipse_arc.safe_rotate_z_origin(angle);
        assert!(result.is_ok());

        // 回転後も有効な楕円弧である
        let rotated = result.unwrap();
        assert!(rotated.is_valid());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let factor = 2.0;

        let result = ellipse_arc.safe_scale_origin(factor);
        assert!(result.is_ok());

        let scaled = result.unwrap();
        let tolerance = 1e-10;
        assert!((scaled.semi_major() - 8.0).abs() < tolerance);
        assert!((scaled.semi_minor() - 4.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let ellipse_arc = create_test_ellipse_arc();

        let result = ellipse_arc.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_axis_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let axis_point = Point3D::origin();
        let axis_direction = Vector3D::unit_z();
        let angle = Angle::from_degrees(45.0);

        let result = ellipse_arc.safe_rotate_axis(axis_point, axis_direction, angle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_rotate_axis_zero_axis_error() {
        let ellipse_arc = create_test_ellipse_arc();
        let axis_point = Point3D::origin();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(45.0);

        let result = ellipse_arc.safe_rotate_axis(axis_point, zero_axis, angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_with_angles_success() {
        let ellipse_arc = create_test_ellipse_arc();
        let new_start = Angle::from_degrees(0.0);
        let new_end = Angle::from_degrees(180.0);

        let result = ellipse_arc.safe_with_angles(new_start, new_end);
        assert!(result.is_ok());

        let modified = result.unwrap();
        let tolerance = 1e-10;
        assert!((modified.start_angle().to_degrees() - 0.0).abs() < tolerance);
        assert!((modified.end_angle().to_degrees() - 180.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_sub_arc_success() {
        let ellipse_arc = create_test_ellipse_arc(); // 45度-135度
        let sub_start = Angle::from_degrees(60.0);
        let sub_end = Angle::from_degrees(120.0);

        let result = ellipse_arc.safe_sub_arc(sub_start, sub_end);
        assert!(result.is_ok());

        let sub_arc = result.unwrap();
        let tolerance = 1e-10;
        assert!((sub_arc.start_angle().to_degrees() - 60.0).abs() < tolerance);
        assert!((sub_arc.end_angle().to_degrees() - 120.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reverse_success() {
        let ellipse_arc = create_test_ellipse_arc();

        let result = ellipse_arc.safe_reverse();
        assert!(result.is_ok());

        let reversed = result.unwrap();
        let tolerance = 1e-10;
        assert!((reversed.start_angle().to_degrees() - 135.0).abs() < tolerance);
        assert!((reversed.end_angle().to_degrees() - 45.0).abs() < tolerance);
    }
}
