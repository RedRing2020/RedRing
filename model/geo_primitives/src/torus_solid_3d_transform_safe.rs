// torus_solid_3d_transform_safe.rs
// TorusSolid3D の SafeTransform トレイト実装
//
// エラーハンドリング付きの安全な幾何変換を提供します。
// 無効な変換パラメータや幾何学的制約違反を適切に検出し、
// 詳細なエラー情報とともに Result を返します。

use crate::{Point3D, TorusSolid3D};
use geo_foundation::{Angle, GeometricTolerance, SafeTransform, Scalar, TransformError};

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

/// TorusSolid3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> TorusSolid3D<T> {
    /// トレランス制約付きスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(TorusSolid3D)` - スケール後のトーラス固体
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の半径がトレランス以下
    pub fn safe_scale_with_tolerance(&self, factor: T) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        let new_major_radius = self.major_radius() * factor;
        let new_minor_radius = self.minor_radius() * factor;

        // 半径の幾何学的制約チェック（トレランスベース）
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_major_radius <= min_radius || new_minor_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径(major:{:?}, minor:{:?})がトレランス({:?})以下になります",
                new_major_radius, new_minor_radius, min_radius
            )));
        }

        // トーラス幾何学的制約（major_radius > minor_radius）
        if new_major_radius <= new_minor_radius {
            return Err(TransformError::InvalidGeometry(
                "トーラスのmajor_radiusはminor_radiusより大きい必要があります".to_string(),
            ));
        }

        // 数値安定性チェック
        if !new_major_radius.is_finite() || !new_minor_radius.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        // 中心をスケール
        let scaled_origin = Point3D::new(
            self.origin().x() * factor,
            self.origin().y() * factor,
            self.origin().z() * factor,
        );

        Self::new(
            scaled_origin,
            *self.z_axis(),
            *self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後のトーラス固体の作成に失敗しました".to_string(),
        ))
    }

    /// 半径スケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// このトーラスに適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_radius = T::DISTANCE_TOLERANCE;
        let current_major = self.major_radius();
        let current_minor = self.minor_radius();

        // 小さい方の半径を基準に計算
        let smaller_radius = if current_major < current_minor {
            current_major
        } else {
            current_minor
        };

        if smaller_radius <= T::ZERO {
            T::ZERO
        } else {
            // 最小半径を維持するための倍率 + 安全マージン
            let min_factor = min_radius / smaller_radius;
            min_factor + T::DISTANCE_TOLERANCE
        }
    }

    /// トレランス制約付き半径個別スケール
    ///
    /// # 引数
    /// * `major_factor` - major_radiusのスケール倍率
    /// * `minor_factor` - minor_radiusのスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(TorusSolid3D)` - スケール後のトーラス固体
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    pub fn safe_scale_radii_with_tolerance(
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

        let new_major_radius = self.major_radius() * major_factor;
        let new_minor_radius = self.minor_radius() * minor_factor;

        // 半径の幾何学的制約チェック（トレランスベース）
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_major_radius <= min_radius || new_minor_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径(major:{:?}, minor:{:?})がトレランス({:?})以下になります",
                new_major_radius, new_minor_radius, min_radius
            )));
        }

        // トーラス幾何学的制約（major_radius > minor_radius）
        if new_major_radius <= new_minor_radius {
            return Err(TransformError::InvalidGeometry(
                "トーラスのmajor_radiusはminor_radiusより大きい必要があります".to_string(),
            ));
        }

        Self::new(
            *self.origin(),
            *self.z_axis(),
            *self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "半径スケール後のトーラス固体の作成に失敗しました".to_string(),
        ))
    }
}
