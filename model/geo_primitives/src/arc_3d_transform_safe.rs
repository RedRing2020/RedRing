//! Arc3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{Arc3D, Direction3D, Point3D, Vector3D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// Arc3Dの安全な変換操作
impl<T: Scalar> Arc3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 移動後の円弧
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

        let new_center = self.center() + translation;

        // 移動結果の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "移動計算結果が無効".to_string(),
            ));
        }

        Self::new(
            new_center,
            self.radius(),
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "移動後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な回転（Z軸中心、原点中心）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 回転後の円弧
    /// * `Err(TransformError)` - 無効な角度（無限大、NaN）
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_z(Point3D::origin(), angle)
    }

    /// 安全な回転（Z軸中心、指定点中心）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 回転後の円弧
    /// * `Err(TransformError)` - 無効な入力（無限大、NaN）
    pub fn safe_rotate_z(
        &self,
        center: Point3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 回転中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
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

        // 中心点をZ軸回転
        let new_center = Self::rotate_point_z(self.center(), center, cos_angle, sin_angle)?;

        // 法線ベクトルをZ軸回転
        let normal_vec = self.normal().as_vector();
        let rotated_normal_vec = Self::rotate_vector_z(normal_vec, cos_angle, sin_angle)?;
        let new_normal = Direction3D::from_vector(rotated_normal_vec).ok_or(
            TransformError::ZeroVector("回転後の法線ベクトルが無効".to_string()),
        )?;

        // 開始方向ベクトルをZ軸回転
        let start_dir_vec = self.start_direction().as_vector();
        let rotated_start_dir_vec = Self::rotate_vector_z(start_dir_vec, cos_angle, sin_angle)?;
        let new_start_direction = Direction3D::from_vector(rotated_start_dir_vec).ok_or(
            TransformError::ZeroVector("回転後の開始方向ベクトルが無効".to_string()),
        )?;

        Self::new(
            new_center,
            self.radius(),
            new_normal,
            new_start_direction,
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "Z軸回転後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な任意軸回転（Rodriguesの公式使用）
    ///
    /// # 引数
    /// * `axis_point` - 回転軸上の点
    /// * `axis_direction` - 回転軸の方向ベクトル
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 回転後の円弧
    /// * `Err(TransformError)` - 無効な入力（ゼロ軸、無限大、NaN）
    pub fn safe_rotate_axis(
        &self,
        axis_point: Point3D<T>,
        axis_direction: Vector3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 軸上の点の有効性チェック
        if !axis_point.x().is_finite() || !axis_point.y().is_finite() || !axis_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な軸上の点".to_string(),
            ));
        }

        // 軸方向ベクトルの有効性チェック
        if !axis_direction.x().is_finite()
            || !axis_direction.y().is_finite()
            || !axis_direction.z().is_finite()
            || axis_direction.magnitude().is_zero()
        {
            return Err(TransformError::ZeroVector(
                "無効な軸方向ベクトル".to_string(),
            ));
        }

        // 角度の有効性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation("無効な角度".to_string()));
        }

        // 軸方向を正規化
        let axis_normalized = axis_direction.normalize();

        // 中心点を軸回転
        let new_center =
            Self::rotate_point_rodrigues(self.center(), axis_point, axis_normalized, angle_rad)?;

        // 法線ベクトルを軸回転
        let normal_vec = self.normal().as_vector();
        let rotated_normal_vec =
            Self::rotate_vector_rodrigues(normal_vec, axis_normalized, angle_rad)?;
        let new_normal = Direction3D::from_vector(rotated_normal_vec).ok_or(
            TransformError::ZeroVector("回転後の法線ベクトルが無効".to_string()),
        )?;

        // 開始方向ベクトルを軸回転
        let start_dir_vec = self.start_direction().as_vector();
        let rotated_start_dir_vec =
            Self::rotate_vector_rodrigues(start_dir_vec, axis_normalized, angle_rad)?;
        let new_start_direction = Direction3D::from_vector(rotated_start_dir_vec).ok_or(
            TransformError::ZeroVector("回転後の開始方向ベクトルが無効".to_string()),
        )?;

        Self::new(
            new_center,
            self.radius(),
            new_normal,
            new_start_direction,
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "任意軸回転後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 3D点のZ軸回転計算
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `center` - 回転中心点
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point_z(
        point: Point3D<T>,
        center: Point3D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Point3D<T>, TransformError> {
        let relative = point - center;
        let rotated_x = relative.x() * cos_angle - relative.y() * sin_angle;
        let rotated_y = relative.x() * sin_angle + relative.y() * cos_angle;
        let rotated_z = relative.z(); // Z座標は変化なし

        // 回転結果の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() || !rotated_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Z軸回転計算結果が無効".to_string(),
            ));
        }

        let result = center + Vector3D::new(rotated_x, rotated_y, rotated_z);
        Ok(result)
    }

    /// 3DベクトルのZ軸回転計算
    ///
    /// # 引数
    /// * `vector` - 回転対象のベクトル
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後のベクトル
    fn rotate_vector_z(
        vector: Vector3D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Vector3D<T>, TransformError> {
        let rotated_x = vector.x() * cos_angle - vector.y() * sin_angle;
        let rotated_y = vector.x() * sin_angle + vector.y() * cos_angle;
        let rotated_z = vector.z(); // Z成分は変化なし

        // 回転結果の有効性チェック
        if !rotated_x.is_finite() || !rotated_y.is_finite() || !rotated_z.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Z軸ベクトル回転計算結果が無効".to_string(),
            ));
        }

        Ok(Vector3D::new(rotated_x, rotated_y, rotated_z))
    }

    /// Rodriguesの公式による任意軸回転（点）
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `axis_point` - 回転軸上の点
    /// * `axis_normalized` - 正規化された回転軸ベクトル
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転後の点
    fn rotate_point_rodrigues(
        point: Point3D<T>,
        axis_point: Point3D<T>,
        axis_normalized: Vector3D<T>,
        angle: T,
    ) -> Result<Point3D<T>, TransformError> {
        let v = point - axis_point;
        let rotated_v = Self::rotate_vector_rodrigues(v, axis_normalized, angle)?;
        Ok(axis_point + rotated_v)
    }

    /// Rodriguesの公式による任意軸回転（ベクトル）
    ///
    /// # 引数
    /// * `vector` - 回転対象のベクトル
    /// * `axis_normalized` - 正規化された回転軸ベクトル
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転後のベクトル
    fn rotate_vector_rodrigues(
        vector: Vector3D<T>,
        axis_normalized: Vector3D<T>,
        angle: T,
    ) -> Result<Vector3D<T>, TransformError> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Rodriguesの公式: v' = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_cross_v = axis_normalized.cross(&vector);
        let k_dot_v = axis_normalized.dot(&vector);

        let one = T::from_f64(1.0);
        let rotated_v = vector * cos_angle
            + k_cross_v * sin_angle
            + axis_normalized * (k_dot_v * (one - cos_angle));

        // 回転結果の有効性チェック
        if !rotated_v.x().is_finite() || !rotated_v.y().is_finite() || !rotated_v.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "Rodrigues回転計算結果が無効".to_string(),
            ));
        }

        Ok(rotated_v)
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - スケール後の円弧
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
    /// * `Ok(Arc3D)` - スケール後の円弧
    /// * `Err(TransformError)` - 無効な入力（0以下倍率、無限大、NaN）
    pub fn safe_scale(&self, center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        // スケール中心の有効性チェック
        if !center.x().is_finite() || !center.y().is_finite() || !center.z().is_finite() {
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
        let scaled_radius = self.radius() * factor;

        // スケール結果の有効性チェック
        if !scaled_center.x().is_finite()
            || !scaled_center.y().is_finite()
            || !scaled_center.z().is_finite()
            || !scaled_radius.is_finite()
            || scaled_radius <= T::ZERO
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(
            scaled_center,
            scaled_radius,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "スケール後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な非均一スケール（原点中心）
    ///
    /// 非均一スケールは円弧を楕円弧に変換するため、このメソッドは意図的にエラーを返す
    ///
    /// # 引数
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Err(TransformError)` - 円弧は非均一スケールできない
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
    /// 非均一スケールは円弧を楕円弧に変換するため、均一スケールのみサポート
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - X/Y/Zスケールが等しい場合のみ
    /// * `Err(TransformError)` - 非均一スケールまたは無効な入力
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
        if scale_x <= T::ZERO
            || scale_y <= T::ZERO
            || scale_z <= T::ZERO
            || !scale_x.is_finite()
            || !scale_y.is_finite()
            || !scale_z.is_finite()
        {
            return Err(TransformError::InvalidScaleFactor(
                "無効なスケール倍率".to_string(),
            ));
        }

        // 均一スケールのみ許可（円弧を保持）
        let tolerance = T::from_f64(1e-10);
        if (scale_x - scale_y).abs() > tolerance || (scale_y - scale_z).abs() > tolerance {
            return Err(TransformError::InvalidGeometry(
                "円弧の非均一スケールは楕円弧になるため非対応".to_string(),
            ));
        }

        // 均一スケールとして処理
        self.safe_scale(center, scale_x)
    }

    /// 安全な反射（平面に対する）
    ///
    /// # 引数
    /// * `plane_point` - 反射平面上の点
    /// * `plane_normal` - 反射平面の法線ベクトル
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 反射後の円弧
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

        // 平面法線ベクトルの有効性チェック
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

        // 中心点を反射
        let reflected_center = Self::reflect_point(self.center(), plane_point, normal_normalized)?;

        // 円弧の法線ベクトルも反射
        let arc_normal_vec = self.normal().as_vector();
        let reflected_normal_vec = Self::reflect_vector(arc_normal_vec, normal_normalized)?;
        let new_normal = Direction3D::from_vector(reflected_normal_vec).ok_or(
            TransformError::ZeroVector("反射後の法線ベクトルが無効".to_string()),
        )?;

        // 開始方向ベクトルも反射
        let start_dir_vec = self.start_direction().as_vector();
        let reflected_start_dir_vec = Self::reflect_vector(start_dir_vec, normal_normalized)?;
        let new_start_direction = Direction3D::from_vector(reflected_start_dir_vec).ok_or(
            TransformError::ZeroVector("反射後の開始方向ベクトルが無効".to_string()),
        )?;

        // 反射により円弧の向きが逆転するため、開始角と終了角を交換し、符号を反転
        let reflected_start = Angle::from_radians(-self.end_angle().to_radians());
        let reflected_end = Angle::from_radians(-self.start_angle().to_radians());

        Self::new(
            reflected_center,
            self.radius(),
            new_normal,
            new_start_direction,
            reflected_start,
            reflected_end,
        )
        .ok_or(TransformError::InvalidGeometry(
            "反射後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 点を平面に対して反射
    ///
    /// # 引数
    /// * `point` - 反射対象の点
    /// * `plane_point` - 反射平面上の点
    /// * `normal_normalized` - 正規化された平面法線ベクトル
    ///
    /// # 戻り値
    /// 反射後の点
    fn reflect_point(
        point: Point3D<T>,
        plane_point: Point3D<T>,
        normal_normalized: Vector3D<T>,
    ) -> Result<Point3D<T>, TransformError> {
        let to_point = point - plane_point;
        let distance_to_plane = to_point.dot(&normal_normalized);
        let two = T::from_f64(2.0);
        let reflected_point = point - normal_normalized * (distance_to_plane * two);

        // 反射結果の有効性チェック
        if !reflected_point.x().is_finite()
            || !reflected_point.y().is_finite()
            || !reflected_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "反射計算結果が無効".to_string(),
            ));
        }

        Ok(reflected_point)
    }

    /// ベクトルを平面に対して反射
    ///
    /// # 引数
    /// * `vector` - 反射対象のベクトル
    /// * `normal_normalized` - 正規化された平面法線ベクトル
    ///
    /// # 戻り値
    /// 反射後のベクトル
    fn reflect_vector(
        vector: Vector3D<T>,
        normal_normalized: Vector3D<T>,
    ) -> Result<Vector3D<T>, TransformError> {
        let dot_product = vector.dot(&normal_normalized);
        let two = T::from_f64(2.0);
        let reflected_vector = vector - normal_normalized * (dot_product * two);

        // 反射結果の有効性チェック
        if !reflected_vector.x().is_finite()
            || !reflected_vector.y().is_finite()
            || !reflected_vector.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "ベクトル反射計算結果が無効".to_string(),
            ));
        }

        Ok(reflected_vector)
    }

    /// 安全な半径のみスケール（中心・方向・角度固定）
    ///
    /// # 引数
    /// * `factor` - 半径のスケール倍率
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 半径スケール後の円弧
    /// * `Err(TransformError)` - 無効なスケール倍率（0以下、無限大、NaN）
    pub fn safe_scale_radius(&self, factor: T) -> Result<Self, TransformError> {
        // スケール倍率の有効性チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効な半径スケール倍率".to_string(),
            ));
        }

        let new_radius = self.radius() * factor;

        // スケール結果の有効性チェック
        if !new_radius.is_finite() || new_radius <= T::ZERO {
            return Err(TransformError::InvalidGeometry(
                "半径スケール計算結果が無効".to_string(),
            ));
        }

        Self::new(
            self.center(),
            new_radius,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "半径スケール後の円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な角度範囲変更（中心・半径・方向固定）
    ///
    /// # 引数
    /// * `new_start_angle` - 新しい開始角度
    /// * `new_end_angle` - 新しい終了角度
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 新しい角度範囲の円弧
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

        Self::new(
            self.center(),
            self.radius(),
            self.normal(),
            self.start_direction(),
            new_start_angle,
            new_end_angle,
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい角度での円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な中心点変更（半径・方向・角度固定）
    ///
    /// # 引数
    /// * `new_center` - 新しい中心点
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 新しい中心の円弧
    /// * `Err(TransformError)` - 無効な中心点（無限大、NaN）
    pub fn safe_with_center(&self, new_center: Point3D<T>) -> Result<Self, TransformError> {
        // 新しい中心点の有効性チェック
        if !new_center.x().is_finite() || !new_center.y().is_finite() || !new_center.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "無効な新しい中心点".to_string(),
            ));
        }

        Self::new(
            new_center,
            self.radius(),
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい中心での円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な法線変更（中心・半径・角度固定）
    ///
    /// # 引数
    /// * `new_normal` - 新しい法線方向
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 新しい法線の円弧
    /// * `Err(TransformError)` - 無効な法線
    pub fn safe_with_normal(&self, new_normal: Direction3D<T>) -> Result<Self, TransformError> {
        Self::new(
            self.center(),
            self.radius(),
            new_normal,
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい法線での円弧の作成に失敗".to_string(),
        ))
    }

    /// 安全な開始方向変更（中心・半径・法線・角度固定）
    ///
    /// # 引数
    /// * `new_start_direction` - 新しい開始方向
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 新しい開始方向の円弧
    /// * `Err(TransformError)` - 無効な開始方向
    pub fn safe_with_start_direction(
        &self,
        new_start_direction: Direction3D<T>,
    ) -> Result<Self, TransformError> {
        Self::new(
            self.center(),
            self.radius(),
            self.normal(),
            new_start_direction,
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "新しい開始方向での円弧の作成に失敗".to_string(),
        ))
    }
}

/// Arc3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> Arc3D<T> {
    /// トレランス制約付き半径スケール（中心・法線・角度固定）
    ///
    /// # 引数
    /// * `factor` - 半径のスケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(Arc3D)` - 半径スケール後の円弧
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後の半径がトレランス以下
    pub fn safe_scale_radius_with_tolerance(&self, factor: T) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "半径スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        let new_radius = self.radius() * factor;

        // 半径の幾何学的制約チェック（トレランスベース）
        let min_radius = T::DISTANCE_TOLERANCE;
        if new_radius <= min_radius {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後の半径({:?})がトレランス({:?})以下になります",
                new_radius, min_radius
            )));
        }

        // 数値安定性チェック
        if !new_radius.is_finite() {
            return Err(TransformError::InvalidGeometry(
                "半径スケール計算結果が無効です".to_string(),
            ));
        }

        Self::new(
            self.center(),
            new_radius,
            self.normal(),
            self.start_direction(),
            self.start_angle(),
            self.end_angle(),
        )
        .ok_or(TransformError::InvalidGeometry(
            "半径スケール後の円弧の作成に失敗しました".to_string(),
        ))
    }

    /// 半径スケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// この円弧に適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_radius = T::DISTANCE_TOLERANCE;
        let current_radius = self.radius();

        if current_radius <= T::ZERO {
            T::ZERO
        } else {
            // 最小半径を維持するための倍率 + 安全マージン
            let min_factor = min_radius / current_radius;
            min_factor + T::DISTANCE_TOLERANCE
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_arc() -> Arc3D<f64> {
        let center = Point3D::new(2.0, 3.0, 4.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        Arc3D::new(
            center,
            5.0,
            normal,
            start_direction,
            Angle::from_degrees(0.0),
            Angle::from_degrees(90.0),
        )
        .unwrap()
    }

    #[test]
    fn test_safe_translate_success() {
        let arc = create_test_arc();
        let translation = Vector3D::new(3.0, 4.0, 5.0);
        let result = arc.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.center().x() - 5.0).abs() < tolerance);
        assert!((result.center().y() - 7.0).abs() < tolerance);
        assert!((result.center().z() - 9.0).abs() < tolerance);
        assert!((result.radius() - 5.0).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 0.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let arc = create_test_arc();

        // 無限大の移動ベクトル
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let result = arc.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector3D::new(f64::NAN, 0.0, 0.0);
        let result = arc.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let center = Point3D::new(3.0, 0.0, 5.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let arc = Arc3D::new(
            center,
            2.0,
            normal,
            start_direction,
            Angle::from_degrees(0.0),
            Angle::from_degrees(90.0),
        )
        .unwrap();

        // 90度Z軸回転
        let result = arc.safe_rotate_z_origin(Angle::from_degrees(90.0)).unwrap();

        let tolerance = 1e-10;
        // (3,0,5) が (0,3,5) に回転
        assert!((result.center().x() - 0.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.center().z() - 5.0).abs() < tolerance);
        assert!((result.radius() - 2.0).abs() < tolerance);

        // 開始方向が (1,0,0) から (0,1,0) に回転
        assert!((result.start_direction().x() - 0.0).abs() < tolerance);
        assert!((result.start_direction().y() - 1.0).abs() < tolerance);
        assert!((result.start_direction().z() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_axis_success() {
        let center = Point3D::new(1.0, 0.0, 0.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        let start_direction = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let arc = Arc3D::new(
            center,
            1.5,
            normal,
            start_direction,
            Angle::from_degrees(0.0),
            Angle::from_degrees(180.0),
        )
        .unwrap();

        // Z軸中心90度回転
        let axis_point = Point3D::origin();
        let axis_direction = Vector3D::new(0.0, 0.0, 1.0);
        let result = arc
            .safe_rotate_axis(axis_point, axis_direction, Angle::from_degrees(90.0))
            .unwrap();

        let tolerance = 1e-10;
        // X軸上の点がY軸上に回転
        assert!((result.center().x() - 0.0).abs() < tolerance);
        assert!((result.center().y() - 1.0).abs() < tolerance);
        assert!((result.center().z() - 0.0).abs() < tolerance);
        assert!((result.radius() - 1.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let arc = create_test_arc();

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = arc.safe_rotate_z_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let arc = create_test_arc();
        let result = arc.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 中心と半径がともに2倍
        assert!((result.center().x() - 4.0).abs() < tolerance);
        assert!((result.center().y() - 6.0).abs() < tolerance);
        assert!((result.center().z() - 8.0).abs() < tolerance);
        assert!((result.radius() - 10.0).abs() < tolerance);
        // 角度は変わらない
        assert!((result.start_angle().to_degrees() - 0.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let arc = create_test_arc();
        let result = arc.safe_scale_origin(0.0);
        assert!(result.is_err());

        let result = arc.safe_scale_origin(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_uniform_case() {
        let arc = create_test_arc();
        // 同じスケール値なら成功
        let result = arc.safe_scale_non_uniform_origin(2.0, 2.0, 2.0).unwrap();

        let tolerance = 1e-10;
        assert!((result.center().x() - 4.0).abs() < tolerance);
        assert!((result.center().y() - 6.0).abs() < tolerance);
        assert!((result.center().z() - 8.0).abs() < tolerance);
        assert!((result.radius() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_non_uniform_error() {
        let arc = create_test_arc();
        // 異なるスケール値はエラー
        let result = arc.safe_scale_non_uniform_origin(2.0, 3.0, 2.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_reflect_success() {
        let center = Point3D::new(3.0, 2.0, 5.0);
        let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let start_direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let arc = Arc3D::new(
            center,
            1.5,
            normal,
            start_direction,
            Angle::from_degrees(0.0),
            Angle::from_degrees(180.0),
        )
        .unwrap();

        // XY平面に対する反射（Z座標が反転）
        let plane_point = Point3D::origin();
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let result = arc.safe_reflect(plane_point, plane_normal).unwrap();

        let tolerance = 1e-10;
        // Z座標が反転される
        assert!((result.center().x() - 3.0).abs() < tolerance);
        assert!((result.center().y() - 2.0).abs() < tolerance);
        assert!((result.center().z() - (-5.0)).abs() < tolerance);
        assert!((result.radius() - 1.5).abs() < tolerance);
        // 角度が反転・交換される
        assert!((result.start_angle().to_degrees() - (-180.0)).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 0.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_radius_success() {
        let arc = create_test_arc();
        let result = arc.safe_scale_radius(1.5).unwrap();

        let tolerance = 1e-10;
        // 中心・方向・角度は変わらず、半径のみ1.5倍
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.center().z() - 4.0).abs() < tolerance);
        assert!((result.radius() - 7.5).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 0.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 90.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_angles_success() {
        let arc = create_test_arc();
        let result = arc
            .safe_with_angles(Angle::from_degrees(45.0), Angle::from_degrees(135.0))
            .unwrap();

        let tolerance = 1e-10;
        // 中心・半径・方向は変わらず、角度のみ変更
        assert!((result.center().x() - 2.0).abs() < tolerance);
        assert!((result.center().y() - 3.0).abs() < tolerance);
        assert!((result.center().z() - 4.0).abs() < tolerance);
        assert!((result.radius() - 5.0).abs() < tolerance);
        assert!((result.start_angle().to_degrees() - 45.0).abs() < tolerance);
        assert!((result.end_angle().to_degrees() - 135.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_with_center_success() {
        let arc = create_test_arc();
        let new_center = Point3D::new(10.0, 15.0, 20.0);
        let result = arc.safe_with_center(new_center).unwrap();

        let tolerance = 1e-10;
        // 半径・方向・角度は変わらず、中心のみ変更
        assert!((result.center().x() - 10.0).abs() < tolerance);
        assert!((result.center().y() - 15.0).abs() < tolerance);
        assert!((result.center().z() - 20.0).abs() < tolerance);
        assert!((result.radius() - 5.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_zero_normal_error() {
        let arc = create_test_arc();
        let plane_point = Point3D::origin();
        let zero_normal = Vector3D::new(0.0, 0.0, 0.0);
        let result = arc.safe_reflect(plane_point, zero_normal);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_axis_zero_axis_error() {
        let arc = create_test_arc();
        let axis_point = Point3D::origin();
        let zero_axis = Vector3D::new(0.0, 0.0, 0.0);
        let result = arc.safe_rotate_axis(axis_point, zero_axis, Angle::from_degrees(90.0));
        assert!(result.is_err());
    }
}
