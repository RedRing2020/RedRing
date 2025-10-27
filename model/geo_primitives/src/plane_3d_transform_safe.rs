//! Plane3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Plane3D, Point3D, Vector3D};
use analysis::Angle;
use geo_foundation::{Scalar, TransformError};

/// Plane3Dの安全な変換操作
impl<T: Scalar> Plane3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - 移動後の平面
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        // 移動ベクトルの有効性チェック
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル（無限大またはNaN）".to_string(),
            ));
        }

        // 移動後の参照点を計算
        let new_point = Point3D::new(
            self.point().x() + translation.x(),
            self.point().y() + translation.y(),
            self.point().z() + translation.z(),
        );

        // 移動後の点の有効性チェック
        if !new_point.x().is_finite() || !new_point.y().is_finite() || !new_point.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "移動後の参照点が無効（無限大またはNaN）".to_string(),
            ));
        }

        // 法線ベクトルは平行移動で変化しないため、そのまま使用
        Ok(Plane3D::from_point_and_normal(new_point, self.normal()).unwrap())
    }

    /// 安全な回転変換（Z軸周りの回転）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - 回転後の平面
    /// * `Err(TransformError)` - 無効な回転パラメータ
    pub fn safe_rotate(&self, center: Point3D<T>, angle: Angle<T>) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転中心が無効（無限大またはNaN）".to_string(),
            ));
        }

        // 角度の有効性チェック
        if !angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度が無効（無限大またはNaN）".to_string(),
            ));
        }

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 三角関数の結果の有効性チェック
        if !cos_a.is_finite() || !sin_a.is_finite() {
            return Err(TransformError::InvalidRotation(
                "角度計算結果が無効".to_string(),
            ));
        }

        // 参照点の回転
        let translated_point = Vector3D::new(
            self.point().x() - center.x(),
            self.point().y() - center.y(),
            self.point().z() - center.z(),
        );

        let rotated_point = Vector3D::new(
            translated_point.x() * cos_a - translated_point.y() * sin_a,
            translated_point.x() * sin_a + translated_point.y() * cos_a,
            translated_point.z(), // Z軸周りの回転なのでZ成分は変わらない
        );

        let new_point = Point3D::new(
            rotated_point.x() + center.x(),
            rotated_point.y() + center.y(),
            rotated_point.z() + center.z(),
        );

        // 回転後の点の有効性チェック
        if !new_point.x().is_finite() || !new_point.y().is_finite() || !new_point.z().is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転後の参照点が無効".to_string(),
            ));
        }

        // 法線ベクトルの回転
        let rotated_normal = Vector3D::new(
            self.normal().x() * cos_a - self.normal().y() * sin_a,
            self.normal().x() * sin_a + self.normal().y() * cos_a,
            self.normal().z(),
        );

        // 回転後の法線ベクトルの有効性チェック
        if !rotated_normal.x().is_finite()
            || !rotated_normal.y().is_finite()
            || !rotated_normal.z().is_finite()
        {
            return Err(TransformError::InvalidRotation(
                "回転後の法線ベクトルが無効".to_string(),
            ));
        }

        // 法線ベクトルの正規化状態確認
        let normal_length_sq = rotated_normal.x() * rotated_normal.x()
            + rotated_normal.y() * rotated_normal.y()
            + rotated_normal.z() * rotated_normal.z();

        if normal_length_sq.is_zero() {
            return Err(TransformError::InvalidRotation(
                "回転後の法線ベクトルがゼロベクトル".to_string(),
            ));
        }

        Ok(Plane3D::from_point_and_normal(new_point, rotated_normal).unwrap())
    }

    /// 安全なスケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール係数
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - スケール後の平面
    /// * `Err(TransformError)` - 無効なスケールパラメータ
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール中心が無効（無限大またはNaN）".to_string(),
            ));
        }

        // スケール係数の有効性チェック
        if !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール係数が無効（無限大またはNaN）".to_string(),
            ));
        }

        if factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール係数がゼロ".to_string(),
            ));
        }

        if factor < T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "負のスケール係数（平面の向きが反転）".to_string(),
            ));
        }

        // 参照点のスケール変換
        let scaled_offset = Vector3D::new(
            (self.point().x() - center.x()) * factor,
            (self.point().y() - center.y()) * factor,
            (self.point().z() - center.z()) * factor,
        );

        let new_point = Point3D::new(
            center.x() + scaled_offset.x(),
            center.y() + scaled_offset.y(),
            center.z() + scaled_offset.z(),
        );

        // スケール後の点の有効性チェック
        if !new_point.x().is_finite() || !new_point.y().is_finite() || !new_point.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール後の参照点が無効".to_string(),
            ));
        }

        // 法線ベクトルはスケール変換で変化しない（単位ベクトルを保持）
        Ok(Plane3D::from_point_and_normal(new_point, self.normal()).unwrap())
    }

    /// 安全な一様スケール変換（XYZ軸すべて同じ係数）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - 一様スケール係数
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - スケール後の平面
    /// * `Err(TransformError)` - 無効なスケールパラメータ
    pub fn safe_uniform_scale(
        &self,
        center: Point3D<T>,
        factor: T,
    ) -> Result<Self, TransformError> {
        self.safe_scale(center, factor)
    }

    /// 安全な非一様スケール変換（XYZ軸で異なる係数）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factors` - 各軸のスケール係数 (x, y, z)
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - スケール後の平面
    /// * `Err(TransformError)` - 無効なスケールパラメータ
    ///
    /// # 注意
    /// 非一様スケールでは法線ベクトルも変形する可能性があります
    pub fn safe_non_uniform_scale(
        &self,
        center: Point3D<T>,
        factors: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール中心が無効（無限大またはNaN）".to_string(),
            ));
        }

        // 各軸のスケール係数の有効性チェック
        if !factors.x().is_finite() || !factors.y().is_finite() || !factors.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール係数が無効（無限大またはNaN）".to_string(),
            ));
        }

        if factors.x().is_zero() || factors.y().is_zero() || factors.z().is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール係数にゼロが含まれています".to_string(),
            ));
        }

        if factors.x() < T::ZERO || factors.y() < T::ZERO || factors.z() < T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "負のスケール係数（座標系の反転）".to_string(),
            ));
        }

        // 参照点のスケール変換
        let scaled_offset = Vector3D::new(
            (self.point().x() - center.x()) * factors.x(),
            (self.point().y() - center.y()) * factors.y(),
            (self.point().z() - center.z()) * factors.z(),
        );

        let new_point = Point3D::new(
            center.x() + scaled_offset.x(),
            center.y() + scaled_offset.y(),
            center.z() + scaled_offset.z(),
        );

        // スケール後の点の有効性チェック
        if !new_point.x().is_finite() || !new_point.y().is_finite() || !new_point.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール後の参照点が無効".to_string(),
            ));
        }

        // 非一様スケールでは法線ベクトルも変形する
        // 逆数を使用して法線の変形を計算
        let inv_factors = Vector3D::new(
            T::ONE / factors.x(),
            T::ONE / factors.y(),
            T::ONE / factors.z(),
        );

        let transformed_normal = Vector3D::new(
            self.normal().x() * inv_factors.x(),
            self.normal().y() * inv_factors.y(),
            self.normal().z() * inv_factors.z(),
        );

        // 変形後の法線ベクトルの有効性チェック
        if !transformed_normal.x().is_finite()
            || !transformed_normal.y().is_finite()
            || !transformed_normal.z().is_finite()
        {
            return Err(TransformError::InvalidScaleFactor(
                "スケール後の法線ベクトルが無効".to_string(),
            ));
        }

        // 法線ベクトルの正規化
        let normal_length_sq = transformed_normal.x() * transformed_normal.x()
            + transformed_normal.y() * transformed_normal.y()
            + transformed_normal.z() * transformed_normal.z();

        if normal_length_sq.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール後の法線ベクトルがゼロベクトル".to_string(),
            ));
        }

        let normal_length = normal_length_sq.sqrt();
        let normalized_normal = Vector3D::new(
            transformed_normal.x() / normal_length,
            transformed_normal.y() / normal_length,
            transformed_normal.z() / normal_length,
        );

        Ok(Plane3D::from_point_and_normal(new_point, normalized_normal).unwrap())
    }

    /// 安全な複合変換（平行移動 + 回転 + スケール）
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    /// * `rotation_center` - 回転中心点
    /// * `rotation_angle` - 回転角度
    /// * `scale_center` - スケール中心点
    /// * `scale_factor` - スケール係数
    ///
    /// # 戻り値
    /// * `Ok(Plane3D)` - 変換後の平面
    /// * `Err(TransformError)` - 無効な変換パラメータ
    pub fn safe_transform_composite(
        &self,
        translation: Vector3D<T>,
        rotation_center: Point3D<T>,
        rotation_angle: Angle<T>,
        scale_center: Point3D<T>,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        // 段階的に変換を適用し、各段階でエラーチェック
        let translated = self.safe_translate(translation)?;
        let rotated = translated.safe_rotate(rotation_center, rotation_angle)?;
        let scaled = rotated.safe_scale(scale_center, scale_factor)?;

        Ok(scaled)
    }
}
