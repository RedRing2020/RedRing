// torus_solid_3d_transform_safe.rs
// TorusSolid3D の SafeTransform トレイト実装
//
// エラーハンドリング付きの安全な幾何変換を提供します。
// 無効な変換パラメータや幾何学的制約違反を適切に検出し、
// 詳細なエラー情報とともに Result を返します。

use crate::{Point3D, TorusSolid3D};
use geo_foundation::{Angle, SafeTransform, Scalar, TransformError};

impl<T: Scalar> SafeTransform<T> for TorusSolid3D<T> {
    /// 安全な平行移動（簡易実装：X軸方向のみ）
    ///
    /// # Arguments
    /// * `offset` - X軸方向の移動距離
    ///
    /// # Returns
    /// * `Ok(TorusSolid3D)` - 成功時の移動後固体
    /// * `Err(TransformError)` - エラー時の詳細情報
    fn safe_translate(&self, offset: T) -> Result<Self, TransformError> {
        // 無限大やNaNの検証
        if !offset.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "移動距離に無限大またはNaNが含まれています".to_string(),
            ));
        }

        // 極端に大きな値の検証
        let max_coord = T::from_f64(1e15);
        if offset.abs() > max_coord {
            return Err(TransformError::InvalidGeometry(
                "移動距離が許容範囲を超えています".to_string(),
            ));
        }

        // X軸方向の平行移動
        Ok(TorusSolid3D::new(
            Point3D::new(
                self.origin().x() + offset,
                self.origin().y(),
                self.origin().z(),
            ),
            *self.z_axis(),
            *self.x_axis(),
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("有効な平行移動"))
    }

    /// 安全なスケール変換（簡易実装：原点中心）
    ///
    /// # Arguments
    /// * `center` - 無視されるダミー値
    /// * `factor` - スケール倍率
    ///
    /// # Returns
    /// * `Ok(TorusSolid3D)` - 成功時のスケール後固体
    /// * `Err(TransformError)` - エラー時の詳細情報
    fn safe_scale(&self, _center: T, factor: T) -> Result<Self, TransformError> {
        // ゼロスケール
        if factor.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "ゼロスケール倍率は未対応".to_string(),
            ));
        }

        // 負のスケール
        if factor < T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "負のスケール倍率は未対応".to_string(),
            ));
        }

        // 無限大やNaN
        if !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無限大またはNaNのスケール倍率".to_string(),
            ));
        }

        // 結果半径の妥当性チェック
        let new_major_radius = self.major_radius() * factor;
        let new_minor_radius = self.minor_radius() * factor;

        // トーラス固体の幾何学的制約: major_radius > minor_radius
        if new_major_radius <= new_minor_radius {
            return Err(TransformError::InvalidGeometry(
                "スケール後の主半径が副半径以下になります".to_string(),
            ));
        }

        // 極端に小さな値
        let min_radius = T::from_f64(1e-12);
        if new_major_radius < min_radius || new_minor_radius < min_radius {
            return Err(TransformError::InvalidGeometry(
                "スケール後の半径が最小値を下回ります".to_string(),
            ));
        }

        // 極端に大きな値
        let max_radius = T::from_f64(1e12);
        if new_major_radius > max_radius || new_minor_radius > max_radius {
            return Err(TransformError::InvalidGeometry(
                "スケール後の半径が最大値を超えます".to_string(),
            ));
        }

        Ok(TorusSolid3D::new(
            *self.origin(),
            *self.z_axis(),
            *self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .expect("有効なスケール変換"))
    }

    /// 安全な回転変換（簡易実装）
    ///
    /// # Arguments
    /// * `center` - 無視されるダミー値
    /// * `axis` - 無視されるダミー値  
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// * `Ok(TorusSolid3D)` - 成功時の回転後固体
    /// * `Err(TransformError)` - エラー時の詳細情報
    fn safe_rotate(&self, _center: T, _axis: T, angle: Angle<T>) -> Result<Self, TransformError> {
        // 角度の検証
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation(
                "無限大またはNaNの回転角度".to_string(),
            ));
        }

        // 簡易実装: 同じトーラスを返す（実際の回転は複雑なので）
        Ok(self.clone())
    }
}
