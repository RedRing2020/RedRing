//! TorusSurface3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! トーラス面の半径や軸の有効性を厳密に検証

use crate::{Direction3D, Point3D, TorusSurface3D, Vector3D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// TorusSurface3Dの安全な変換操作
impl<T: Scalar> TorusSurface3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 移動後のトーラス面
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

        // 移動結果の有効性チェック
        if !new_origin.x().is_finite() || !new_origin.y().is_finite() || !new_origin.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "移動計算結果が無効".to_string(),
            ));
        }

        Self::new(
            new_origin,
            self.z_axis(),
            self.x_axis(),
            self.major_radius(),
            self.minor_radius(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "移動後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全なスケール変換
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向スケール倍率
    /// * `scale_y` - Y方向スケール倍率
    /// * `scale_z` - Z方向スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - スケール後のトーラス面
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    pub fn safe_scale(
        &self,
        center: Point3D<T>,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if !scale_x.is_finite() || !scale_y.is_finite() || !scale_z.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        if scale_x.is_zero() || scale_y.is_zero() || scale_z.is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "ゼロスケール倍率は未対応".to_string(),
            ));
        }

        if scale_x < T::ZERO || scale_y < T::ZERO || scale_z < T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "負のスケール倍率は未対応".to_string(),
            ));
        }

        // 中心点の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効なスケール中心点".to_string(),
            ));
        }

        // 原点のスケール変換
        let scaled_origin = Point3D::new(
            center.x() + (self.origin().x() - center.x()) * scale_x,
            center.y() + (self.origin().y() - center.y()) * scale_y,
            center.z() + (self.origin().z() - center.z()) * scale_z,
        );

        // 結果の有効性チェック
        if !scaled_origin.x().is_finite()
            || !scaled_origin.y().is_finite()
            || !scaled_origin.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効".to_string(),
            ));
        }

        // トーラス面では、主半径と副半径は異方性スケールでは複雑になるため、
        // 等方性スケールのみサポート（全軸同じ倍率）
        if (scale_x - scale_y).abs() > T::EPSILON || (scale_y - scale_z).abs() > T::EPSILON {
            return Err(TransformError::InvalidScaleFactor(
                "トーラス面では等方性スケールのみサポート".to_string(),
            ));
        }

        let scale_factor = scale_x;
        let new_major_radius = self.major_radius() * scale_factor;
        let new_minor_radius = self.minor_radius() * scale_factor;

        // 半径の有効性チェック
        if !new_major_radius.is_finite()
            || !new_minor_radius.is_finite()
            || new_major_radius <= T::ZERO
            || new_minor_radius <= T::ZERO
        {
            return Err(TransformError::InvalidGeometry(
                "スケール後の半径が無効".to_string(),
            ));
        }

        // トーラス面の幾何的制約チェック（major_radius > minor_radius）
        if new_major_radius <= new_minor_radius {
            return Err(TransformError::InvalidGeometry(
                "スケール後の主半径が副半径以下".to_string(),
            ));
        }

        Self::new(
            scaled_origin,
            self.z_axis(),
            self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全な回転変換（原点中心、Z軸回転）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 回転後のトーラス面
    /// * `Err(TransformError)` - 無効な回転角度または計算結果
    pub fn safe_rotate_z(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation(
                "無効な回転角度".to_string(),
            ));
        }

        // 回転計算
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 三角関数の有効性チェック
        if !cos_angle.is_finite() || !sin_angle.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転計算結果が無効".to_string(),
            ));
        }

        // 原点の回転
        let new_origin = Point3D::new(
            self.origin().x() * cos_angle - self.origin().y() * sin_angle,
            self.origin().x() * sin_angle + self.origin().y() * cos_angle,
            self.origin().z(),
        );

        // X軸の回転
        let rotated_x_vector = Vector3D::new(
            self.x_axis().x() * cos_angle - self.x_axis().y() * sin_angle,
            self.x_axis().x() * sin_angle + self.x_axis().y() * cos_angle,
            self.x_axis().z(),
        );

        let new_x_axis = Direction3D::from_vector(rotated_x_vector).ok_or(
            TransformError::InvalidRotation("回転後のX軸が無効".to_string()),
        )?;

        // 結果の有効性チェック
        if !new_origin.x().is_finite() || !new_origin.y().is_finite() || !new_origin.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        Self::new(
            new_origin,
            self.z_axis(),
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "回転後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全な任意軸回転（指定中心、指定軸）
    ///
    /// # 引数
    /// * `center` - 回転中心
    /// * `axis` - 回転軸方向
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 回転後のトーラス面
    /// * `Err(TransformError)` - 無効なパラメータまたは計算結果
    pub fn safe_rotate_around_axis(
        &self,
        center: Point3D<T>,
        axis: Direction3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // パラメータの有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation(
                "無効な回転角度".to_string(),
            ));
        }

        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "無効な回転中心".to_string(),
            ));
        }

        // Rodrigues' rotation formula による回転
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();
        let one_minus_cos = T::ONE - cos_angle;

        if !cos_angle.is_finite() || !sin_angle.is_finite() || !one_minus_cos.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転計算パラメータが無効".to_string(),
            ));
        }

        let k = Vector3D::new(axis.x(), axis.y(), axis.z());

        // 原点を中心基準に移動
        let relative_origin = self.origin().to_vector() - center.to_vector();

        // Rodrigues' formula で原点を回転
        let rotated_origin_vec = relative_origin * cos_angle
            + k.cross(&relative_origin) * sin_angle
            + k * k.dot(&relative_origin) * one_minus_cos;

        let new_origin = center + rotated_origin_vec;

        // X軸を回転
        let x_vec = Vector3D::new(self.x_axis().x(), self.x_axis().y(), self.x_axis().z());
        let rotated_x_vec =
            x_vec * cos_angle + k.cross(&x_vec) * sin_angle + k * k.dot(&x_vec) * one_minus_cos;

        let new_x_axis = Direction3D::from_vector(rotated_x_vec).ok_or(
            TransformError::InvalidRotation("回転後のX軸が無効".to_string()),
        )?;

        // Z軸を回転
        let z_vec = Vector3D::new(self.z_axis().x(), self.z_axis().y(), self.z_axis().z());
        let rotated_z_vec =
            z_vec * cos_angle + k.cross(&z_vec) * sin_angle + k * k.dot(&z_vec) * one_minus_cos;

        let new_z_axis = Direction3D::from_vector(rotated_z_vec).ok_or(
            TransformError::InvalidRotation("回転後のZ軸が無効".to_string()),
        )?;

        // 結果の有効性チェック
        if !new_origin.x().is_finite() || !new_origin.y().is_finite() || !new_origin.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        Self::new(
            new_origin,
            new_z_axis,
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "回転後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全な半径変更
    ///
    /// # 引数
    /// * `new_major_radius` - 新しい主半径
    /// * `new_minor_radius` - 新しい副半径
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 半径変更後のトーラス面
    /// * `Err(TransformError)` - 無効な半径値
    pub fn safe_resize(
        &self,
        new_major_radius: T,
        new_minor_radius: T,
    ) -> Result<Self, TransformError> {
        // 半径の有効性チェック
        if !new_major_radius.is_finite() || !new_minor_radius.is_finite() {
            return Err(TransformError::InvalidGeometry("無効な半径値".to_string()));
        }

        if new_major_radius <= T::ZERO || new_minor_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "半径は正の値である必要があります".to_string(),
            ));
        }

        // トーラス面の幾何的制約チェック
        if new_major_radius <= new_minor_radius {
            return Err(TransformError::InvalidGeometry(
                "主半径は副半径より大きい必要があります".to_string(),
            ));
        }

        Self::new(
            self.origin(),
            self.z_axis(),
            self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "半径変更後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全な軸方向変更
    ///
    /// # 引数
    /// * `new_z_axis` - 新しいZ軸方向
    /// * `new_x_axis` - 新しいX軸方向
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 軸変更後のトーラス面
    /// * `Err(TransformError)` - 無効な軸方向（非直交など）
    pub fn safe_reorient(
        &self,
        new_z_axis: Direction3D<T>,
        new_x_axis: Direction3D<T>,
    ) -> Result<Self, TransformError> {
        // 軸の直交性チェック
        let dot_product = new_z_axis.x() * new_x_axis.x()
            + new_z_axis.y() * new_x_axis.y()
            + new_z_axis.z() * new_x_axis.z();

        if dot_product.abs() > T::EPSILON {
            return Err(TransformError::InvalidGeometry(
                "Z軸とX軸は直交している必要があります".to_string(),
            ));
        }

        Self::new(
            self.origin(),
            new_z_axis,
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "軸変更後のトーラス面の作成に失敗".to_string(),
        ))
    }

    /// 安全なトーラス面の複合変換
    ///
    /// # 引数
    /// * `translation` - 平行移動ベクトル
    /// * `rotation_center` - 回転中心
    /// * `rotation_axis` - 回転軸
    /// * `rotation_angle` - 回転角度
    /// * `scale_factor` - スケール倍率（等方性）
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - 変換後のトーラス面
    /// * `Err(TransformError)` - 無効なパラメータまたは計算結果
    pub fn safe_transform(
        &self,
        translation: Vector3D<T>,
        rotation_center: Point3D<T>,
        rotation_axis: Direction3D<T>,
        rotation_angle: Angle<T>,
        scale_factor: T,
    ) -> Result<Self, TransformError> {
        // 1. スケール変換
        let scaled = self.safe_scale(rotation_center, scale_factor, scale_factor, scale_factor)?;

        // 2. 回転変換
        let rotated =
            scaled.safe_rotate_around_axis(rotation_center, rotation_axis, rotation_angle)?;

        // 3. 平行移動
        rotated.safe_translate(translation)
    }
}

/// TorusSurface3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> TorusSurface3D<T> {
    /// トレランス制約付きスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(TorusSurface3D)` - スケール後のトーラス面
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
            self.z_axis(),
            self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後のトーラス面の作成に失敗しました".to_string(),
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
}
