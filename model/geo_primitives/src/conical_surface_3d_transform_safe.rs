//! ConicalSurface3D の安全な変換操作（SafeTransform）実装
//!
//! エラーハンドリング付きの変換操作を提供

use crate::{ConicalSurface3D, Point3D, Vector3D};
use geo_foundation::{BasicTransform, Scalar, TransformError};

/// 安全な変換操作を提供するトレイト（ConicalSurface3D専用）
pub trait ConicalSurfaceSafeTransform<T: Scalar> {
    /// 安全な平行移動
    fn translate_safe(&self, translation: Vector3D<T>) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全なZ軸周りの回転
    fn rotate_z_safe(&self, angle: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全な任意軸周りの回転
    fn rotate_axis_safe(&self, axis: Vector3D<T>, angle: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全な一様スケール
    fn scale_uniform_safe(&self, factor: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全な非一様スケール
    fn scale_non_uniform_safe(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全な平面による反射
    fn reflect_safe(
        &self,
        plane_point: Point3D<T>,
        plane_normal: Vector3D<T>,
    ) -> Result<Self, TransformError>
    where
        Self: Sized;
}

impl<T: Scalar> ConicalSurfaceSafeTransform<T> for ConicalSurface3D<T> {
    /// 安全な平行移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 成功時は移動後の円錐サーフェス、失敗時はエラー
    fn translate_safe(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        // 平行移動は常に安全
        Ok(self.translate(translation))
    }

    /// 安全なZ軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 成功時は回転後の円錐サーフェス、失敗時はエラー
    fn rotate_z_safe(&self, angle: T) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        if !angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Ok(self.rotate_z(angle))
    }

    /// 安全な任意軸周りの回転
    ///
    /// # Arguments
    /// * `axis` - 回転軸
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 成功時は回転後の円錐サーフェス、失敗時はエラー
    fn rotate_axis_safe(&self, axis: Vector3D<T>, angle: T) -> Result<Self, TransformError> {
        // 回転軸の検証
        if axis.is_zero() {
            return Err(TransformError::ZeroVector(
                "回転軸がゼロベクトルです".to_string(),
            ));
        }

        // 角度の有効性チェック
        if !angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Ok(self.rotate_axis(axis, angle))
    }

    /// 安全な一様スケール
    ///
    /// # Arguments
    /// * `factor` - スケール倍率
    ///
    /// # Returns
    /// 成功時はスケール後の円錐サーフェス、失敗時はエラー
    fn scale_uniform_safe(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の検証
        if factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率がゼロです".to_string(),
            ));
        }

        if !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率が無効です（無限大またはNaN）".to_string(),
            ));
        }

        let result = self.scale_uniform(factor);

        // 結果の有効性チェック
        if !result.is_valid() {
            return Err(TransformError::InvalidGeometry(
                "スケール後の円錐サーフェスが無効です".to_string(),
            ));
        }

        Ok(result)
    }

    /// 安全な非一様スケール
    ///
    /// # Arguments
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # Returns
    /// 成功時はスケール後の円錐サーフェス、失敗時はエラー
    fn scale_non_uniform_safe(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // スケール倍率の検証
        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率にゼロが含まれています".to_string(),
            ));
        }

        if !scale_x.is_finite() || !scale_y.is_finite() || !scale_z.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率が無効です（無限大またはNaN）".to_string(),
            ));
        }

        let result = self.scale_non_uniform(scale_x, scale_y, scale_z);

        // 結果の有効性チェック
        if !result.is_valid() {
            return Err(TransformError::InvalidGeometry(
                "非一様スケール後の円錐サーフェスが無効です".to_string(),
            ));
        }

        Ok(result)
    }

    /// 安全な平面による反射
    ///
    /// # Arguments
    /// * `plane_point` - 平面上の点
    /// * `plane_normal` - 平面の法線ベクトル
    ///
    /// # Returns
    /// 成功時は反射後の円錐サーフェス、失敗時はエラー
    fn reflect_safe(
        &self,
        plane_point: Point3D<T>,
        plane_normal: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        // 平面法線の検証
        if plane_normal.is_zero() {
            return Err(TransformError::ZeroVector(
                "平面の法線ベクトルがゼロです".to_string(),
            ));
        }

        // 平面点の有効性チェック
        if !plane_point.x().is_finite()
            || !plane_point.y().is_finite()
            || !plane_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "平面上の点が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Ok(self.reflect(plane_point, plane_normal))
    }
}

// ============================================================================
// 追加の安全な操作
// ============================================================================

impl<T: Scalar> ConicalSurface3D<T> {
    /// 安全な半径変更
    ///
    /// # Arguments
    /// * `new_radius` - 新しい半径
    ///
    /// # Returns
    /// 成功時は新しい円錐サーフェス、失敗時はエラー
    pub fn with_radius_safe(&self, new_radius: T) -> Result<Self, TransformError> {
        if new_radius <= T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "半径は正の値である必要があります".to_string(),
            ));
        }

        if !new_radius.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "半径が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            self.semi_angle(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("半径変更後の円錐サーフェスが無効です".to_string())
        })
    }

    /// 安全な半頂角変更
    ///
    /// # Arguments
    /// * `new_semi_angle` - 新しい半頂角（ラジアン）
    ///
    /// # Returns
    /// 成功時は新しい円錐サーフェス、失敗時はエラー
    pub fn with_semi_angle_safe(&self, new_semi_angle: T) -> Result<Self, TransformError> {
        let half_pi = T::PI / (T::ONE + T::ONE);

        if new_semi_angle <= T::ZERO || new_semi_angle >= half_pi {
            return Err(TransformError::InvalidRotation(
                "半頂角は 0 < angle < π/2 の範囲である必要があります".to_string(),
            ));
        }

        if !new_semi_angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "半頂角が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            new_semi_angle,
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("半頂角変更後の円錐サーフェスが無効です".to_string())
        })
    }

    /// 安全な軸方向変更
    ///
    /// # Arguments
    /// * `new_axis` - 新しい軸方向
    ///
    /// # Returns
    /// 成功時は新しい円錐サーフェス、失敗時はエラー
    pub fn with_axis_safe(&self, new_axis: Vector3D<T>) -> Result<Self, TransformError> {
        if new_axis.is_zero() {
            return Err(TransformError::ZeroVector(
                "軸方向ベクトルがゼロです".to_string(),
            ));
        }

        let axis_direction = crate::Direction3D::from_vector(new_axis).ok_or_else(|| {
            TransformError::ZeroVector("軸方向の正規化に失敗しました".to_string())
        })?;

        // グラム・シュミット正規直交化で参照方向を調整
        let ref_vec = self.ref_direction().as_vector();
        let axis_vec = axis_direction.as_vector();
        let dot_product =
            ref_vec.x() * axis_vec.x() + ref_vec.y() * axis_vec.y() + ref_vec.z() * axis_vec.z();

        let orthogonal_ref = Vector3D::new(
            ref_vec.x() - dot_product * axis_vec.x(),
            ref_vec.y() - dot_product * axis_vec.y(),
            ref_vec.z() - dot_product * axis_vec.z(),
        );

        let ref_direction = crate::Direction3D::from_vector(orthogonal_ref).ok_or_else(|| {
            TransformError::InvalidGeometry("参照方向の調整に失敗しました".to_string())
        })?;

        Self::new(
            self.center(),
            axis_direction.as_vector(),
            ref_direction.as_vector(),
            self.radius(),
            self.semi_angle(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("軸方向変更後の円錐サーフェスが無効です".to_string())
        })
    }

    /// 安全な参照方向変更
    ///
    /// # Arguments
    /// * `new_ref_direction` - 新しい参照方向
    ///
    /// # Returns
    /// 成功時は新しい円錐サーフェス、失敗時はエラー
    pub fn with_ref_direction_safe(
        &self,
        new_ref_direction: Vector3D<T>,
    ) -> Result<Self, TransformError> {
        if new_ref_direction.is_zero() {
            return Err(TransformError::ZeroVector(
                "参照方向ベクトルがゼロです".to_string(),
            ));
        }

        // グラム・シュミット正規直交化
        let axis_vec = self.axis().as_vector();
        let dot_product = new_ref_direction.x() * axis_vec.x()
            + new_ref_direction.y() * axis_vec.y()
            + new_ref_direction.z() * axis_vec.z();

        let orthogonal_ref = Vector3D::new(
            new_ref_direction.x() - dot_product * axis_vec.x(),
            new_ref_direction.y() - dot_product * axis_vec.y(),
            new_ref_direction.z() - dot_product * axis_vec.z(),
        );

        let ref_direction = crate::Direction3D::from_vector(orthogonal_ref).ok_or_else(|| {
            TransformError::InvalidGeometry("参照方向が軸と平行すぎて正規化できません".to_string())
        })?;

        Self::new(
            self.center(),
            self.axis().as_vector(),
            ref_direction.as_vector(),
            self.radius(),
            self.semi_angle(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("参照方向変更後の円錐サーフェスが無効です".to_string())
        })
    }

    /// 安全な中心点変更
    ///
    /// # Arguments
    /// * `new_center` - 新しい中心点
    ///
    /// # Returns
    /// 成功時は新しい円錐サーフェス、失敗時はエラー
    pub fn with_center_safe(&self, new_center: Point3D<T>) -> Result<Self, TransformError> {
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "中心点が無効です（無限大またはNaN）".to_string(),
            ));
        }

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.semi_angle(),
        )
        .ok_or_else(|| {
            TransformError::InvalidGeometry("中心点変更後の円錐サーフェスが無効です".to_string())
        })
    }
}
