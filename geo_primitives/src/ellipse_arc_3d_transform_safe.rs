//! EllipseArc3D Safe Transform operations
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! geo_foundationのAngle型を使用した型安全なインターフェース

use crate::{EllipseArc3D, Vector3D};
use geo_foundation::{Angle, Scalar, TransformError};

/// EllipseArc3Dの安全な変換操作
impl<T: Scalar> EllipseArc3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 移動後の楕円弧
    /// * `Err(TransformError)` - 無効な移動ベクトル（無限大、NaN）
    pub fn safe_translate(&self, translation: Vector3D<T>) -> Result<Self, TransformError> {
        if !translation.x().is_finite()
            || !translation.y().is_finite()
            || !translation.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な移動ベクトル".to_string(),
            ));
        }
        Ok(self.translate(translation))
    }

    /// 安全な原点中心の均等スケール
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - スケール後の楕円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale(&self, factor: T) -> Result<Self, TransformError> {
        if !factor.is_finite() || factor <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール倍率".to_string(),
            ));
        }
        self.scale(factor)
            .ok_or_else(|| TransformError::InvalidGeometry("スケール操作失敗".to_string()))
    }

    /// 安全なZ軸回転（原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(EllipseArc3D)` - 回転後の楕円弧
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_z(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        if !angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation(
                "無効な回転角度".to_string(),
            ));
        }
        self.rotate_z(angle.to_radians())
            .ok_or_else(|| TransformError::InvalidGeometry("回転操作失敗".to_string()))
    }
}
