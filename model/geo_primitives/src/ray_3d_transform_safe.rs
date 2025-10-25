//! Ray3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Point3D, Ray3D, Vector3D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// Ray3Dの安全な変換操作
impl<T: Scalar> Ray3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 移動後の半無限直線
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

        let new_origin = self.origin() + translation;
        Ok(Self::new(new_origin, self.direction().as_vector()).unwrap())
    }

    /// 安全な回転（原点中心、X軸周り）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_x_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_x(Point3D::origin(), angle)
    }

    /// 安全な回転（指定点中心、X軸周り）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate_x(
        &self,
        center: Point3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        self.safe_rotate_axis(center, Vector3D::unit_x(), angle)
    }

    /// 安全な回転（原点中心、Y軸周り）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_y_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_y(Point3D::origin(), angle)
    }

    /// 安全な回転（指定点中心、Y軸周り）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate_y(
        &self,
        center: Point3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        self.safe_rotate_axis(center, Vector3D::unit_y(), angle)
    }

    /// 安全な回転（原点中心、Z軸周り）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_z(Point3D::origin(), angle)
    }

    /// 安全な回転（指定点中心、Z軸周り）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate_z(
        &self,
        center: Point3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        self.safe_rotate_axis(center, Vector3D::unit_z(), angle)
    }

    /// 安全な回転（任意軸周り）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `axis` - 回転軸ベクトル
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 回転後の半無限直線
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN、ゼロ軸）
    pub fn safe_rotate_axis(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な回転中心".to_string(),
            ));
        }

        // 回転軸の有効性チェック
        if !axis.x().is_finite()
            || !axis.y().is_finite()
            || !axis.z().is_finite()
            || axis.magnitude().is_zero()
        {
            return Err(TransformError::ZeroVector("無効な回転軸".to_string()));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        // 回転軸を正規化
        let axis_normalized = axis.normalize();

        // Rodriguesの回転公式を使用
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 起点を回転
        let relative_origin = self.origin() - center;
        let rotated_origin =
            Self::rodrigues_rotation(relative_origin, axis_normalized, cos_angle, sin_angle);
        let new_origin = center + rotated_origin;

        // 方向ベクトルを回転
        let dir = self.direction().as_vector();
        let rotated_direction =
            Self::rodrigues_rotation(dir, axis_normalized, cos_angle, sin_angle);

        Ok(Self::new(new_origin, rotated_direction).unwrap())
    }

    /// Rodriguesの回転公式による3Dベクトル回転
    ///
    /// # 引数
    /// * `v` - 回転対象ベクトル
    /// * `k` - 正規化された回転軸
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後のベクトル
    fn rodrigues_rotation(
        v: Vector3D<T>,
        k: Vector3D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Vector3D<T> {
        // v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_dot_v = k.dot(&v);
        let k_cross_v = k.cross(&v);
        let one_minus_cos = T::ONE - cos_angle;

        v * cos_angle + k_cross_v * sin_angle + k * k_dot_v * one_minus_cos
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - スケール後の半無限直線
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
    /// * `Ok(Ray3D)` - スケール後の半無限直線
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

        let relative_origin = Vector3D::from_points(&center, &self.origin());
        let scaled_origin = relative_origin * factor;
        let new_origin = center + scaled_origin;

        // 負のスケールは方向を反転させる
        let new_direction = if factor < T::ZERO {
            -self.direction().as_vector()
        } else {
            self.direction().as_vector()
        };

        Ok(Self::new(new_origin, new_direction).unwrap())
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - スケール後の半無限直線
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
    /// * `Ok(Ray3D)` - スケール後の半無限直線
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

        let relative_origin = Vector3D::from_points(&center, &self.origin());
        let scaled_origin = Vector3D::new(
            relative_origin.x() * scale_x,
            relative_origin.y() * scale_y,
            relative_origin.z() * scale_z,
        );
        let new_origin = center + scaled_origin;

        // 方向ベクトルも非均一スケールの影響を受ける
        let dir = self.direction();
        let scaled_direction =
            Vector3D::new(dir.x() * scale_x, dir.y() * scale_y, dir.z() * scale_z);

        // 新しい方向ベクトルでRayを作成
        Self::new(new_origin, scaled_direction).ok_or(TransformError::ZeroVector(
            "スケール後の方向ベクトルがゼロ".to_string(),
        ))
    }

    /// 安全な反射（平面に対する）
    ///
    /// # 引数
    /// * `plane_point` - 反射平面上の点
    /// * `plane_normal` - 反射平面の法線ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Ray3D)` - 反射後の半無限直線
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

        // 法線ベクトルの有効性チェック
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

        // 起点を反射
        let to_origin = Vector3D::from_points(&plane_point, &self.origin());
        let distance_to_plane = to_origin.dot(&normal_normalized);
        let two = T::ONE + T::ONE;
        let reflected_origin = self.origin() - normal_normalized * (distance_to_plane * two);

        // 方向ベクトルを反射
        let dir = self.direction().as_vector();
        let dir_dot_normal = dir.dot(&normal_normalized);
        let reflected_direction = dir - normal_normalized * (dir_dot_normal * two);

        Ok(Self::new(reflected_origin, reflected_direction).unwrap())
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
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let translation = Vector3D::new(3.0, 4.0, 5.0);
        let result = ray.safe_translate(translation).unwrap();

        let expected_origin = Point3D::new(4.0, 6.0, 8.0);
        let tolerance = 1e-10;
        assert!((result.origin().x() - expected_origin.x()).abs() < tolerance);
        assert!((result.origin().y() - expected_origin.y()).abs() < tolerance);
        assert!((result.origin().z() - expected_origin.z()).abs() < tolerance);

        // 方向は変わらない
        assert!((result.direction().x() - ray.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - ray.direction().y()).abs() < tolerance);
        assert!((result.direction().z() - ray.direction().z()).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // 無限大の移動ベクトル
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let result = ray.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector3D::new(f64::NAN, 0.0, 0.0);
        let result = ray.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // Z軸周りに90度回転
        let result = ray.safe_rotate_z_origin(Angle::from_degrees(90.0)).unwrap();

        let tolerance = 1e-10;
        // 起点 (1,0,0) が (0,1,0) に回転
        assert!((result.origin().x() - 0.0).abs() < tolerance);
        assert!((result.origin().y() - 1.0).abs() < tolerance);
        assert!((result.origin().z() - 0.0).abs() < tolerance);

        // 方向 (1,0,0) が (0,1,0) に回転
        assert!((result.direction().x() - 0.0).abs() < tolerance);
        assert!((result.direction().y() - 1.0).abs() < tolerance);
        assert!((result.direction().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_x_origin_success() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(0.0, 1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        // X軸周りに90度回転
        let result = ray.safe_rotate_x_origin(Angle::from_degrees(90.0)).unwrap();

        let tolerance = 1e-10;
        // 起点 (0,1,0) が (0,0,1) に回転
        assert!((result.origin().x() - 0.0).abs() < tolerance);
        assert!((result.origin().y() - 0.0).abs() < tolerance);
        assert!((result.origin().z() - 1.0).abs() < tolerance);

        // 方向 (0,1,0) が (0,0,1) に回転
        assert!((result.direction().x() - 0.0).abs() < tolerance);
        assert!((result.direction().y() - 0.0).abs() < tolerance);
        assert!((result.direction().z() - 1.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = ray.safe_rotate_z_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(2.0, 3.0, 4.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = ray.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 起点がスケールされる
        assert!((result.origin().x() - 4.0).abs() < tolerance);
        assert!((result.origin().y() - 6.0).abs() < tolerance);
        assert!((result.origin().z() - 8.0).abs() < tolerance);

        // 方向は変わらない（正のスケール）
        assert!((result.direction().x() - ray.direction().x()).abs() < tolerance);
        assert!((result.direction().y() - ray.direction().y()).abs() < tolerance);
        assert!((result.direction().z() - ray.direction().z()).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_negative_factor() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(2.0, 3.0, 4.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = ray.safe_scale_origin(-2.0).unwrap();

        let tolerance = 1e-10;
        // 起点がスケールされる
        assert!((result.origin().x() - (-4.0)).abs() < tolerance);
        assert!((result.origin().y() - (-6.0)).abs() < tolerance);
        assert!((result.origin().z() - (-8.0)).abs() < tolerance);

        // 方向が反転される（負のスケール）
        assert!((result.direction().x() - (-ray.direction().x())).abs() < tolerance);
        assert!((result.direction().y() - (-ray.direction().y())).abs() < tolerance);
        assert!((result.direction().z() - (-ray.direction().z())).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = ray.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(2.0, 3.0, 4.0), Vector3D::new(1.0, 1.0, 1.0)).unwrap();

        let result = ray.safe_scale_non_uniform_origin(2.0, 3.0, 4.0).unwrap();

        let tolerance = 1e-10;
        // 起点が非均一スケールされる
        assert!((result.origin().x() - 4.0).abs() < tolerance);
        assert!((result.origin().y() - 9.0).abs() < tolerance);
        assert!((result.origin().z() - 16.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(1.0, 1.0, 1.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        // XY平面に対する反射（Z=0平面、法線は(0,0,1)）
        let plane_point = Point3D::origin();
        let plane_normal = Vector3D::unit_z();
        let result = ray.safe_reflect(plane_point, plane_normal).unwrap();

        let tolerance = 1e-10;
        // 起点 (1,1,1) が (1,1,-1) に反射
        assert!((result.origin().x() - 1.0).abs() < tolerance);
        assert!((result.origin().y() - 1.0).abs() < tolerance);
        assert!((result.origin().z() - (-1.0)).abs() < tolerance);

        // 方向 (1,0,0) は変わらない（法線に垂直）
        assert!((result.direction().x() - 1.0).abs() < tolerance);
        assert!((result.direction().y() - 0.0).abs() < tolerance);
        assert!((result.direction().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_rodrigues_rotation_x_axis() {
        let ray =
            Ray3D::<f64>::new(Point3D::new(0.0, 1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        // X軸周りに90度回転（任意軸回転を使用）
        let result = ray
            .safe_rotate_axis(
                Point3D::origin(),
                Vector3D::unit_x(),
                Angle::from_degrees(90.0),
            )
            .unwrap();

        let tolerance = 1e-10;
        // Y軸上の点がZ軸に回転
        assert!((result.origin().x() - 0.0).abs() < tolerance);
        assert!((result.origin().y() - 0.0).abs() < tolerance);
        assert!((result.origin().z() - 1.0).abs() < tolerance);

        // 方向も同様に回転
        assert!((result.direction().x() - 0.0).abs() < tolerance);
        assert!((result.direction().y() - 0.0).abs() < tolerance);
        assert!((result.direction().z() - 1.0).abs() < tolerance);
    }
}
