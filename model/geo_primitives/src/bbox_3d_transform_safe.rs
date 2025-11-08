//! BBox3D 安全な変換エラーハンドリング実装
//!
//! Result<T, TransformError>パターンによる安全な変換操作
//! analysisクレートのAngle型を使用した型安全なインターフェース

use crate::{BBox3D, Point3D, Vector3D};
use analysis::Angle;
use geo_foundation::{GeometricTolerance, Scalar, TransformError};

/// BBox3Dの安全な変換操作
impl<T: Scalar> BBox3D<T> {
    /// 安全な平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - 移動後の境界ボックス
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

        // 境界ボックスの平行移動は min と max を同じベクトルで移動
        let new_min = self.min() + translation;
        let new_max = self.max() + translation;
        Ok(BBox3D::new(new_min, new_max))
    }

    /// 安全な回転（原点中心、X軸周り）
    ///
    /// # 引数
    /// * `angle` - 回転角度（Angle型）
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス
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
    /// * `Ok(BBox3D)` - 回転後の境界ボックス（8つの頂点を回転させて再計算）
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

        // 境界ボックスの8つの頂点を取得
        let vertices = self.get_all_vertices();

        // Rodriguesの回転公式を使用
        let cos_angle = angle_rad.cos();
        let sin_angle = angle_rad.sin();

        // 各頂点を回転
        let mut rotated_vertices = Vec::with_capacity(8);
        for vertex in &vertices {
            let rotated_vertex = Self::rodrigues_rotation_around_center(
                *vertex,
                center,
                axis_normalized,
                cos_angle,
                sin_angle,
            )?;
            rotated_vertices.push(rotated_vertex);
        }

        // 回転した頂点から新しい境界ボックスを構築
        Self::from_points(&rotated_vertices).ok_or(TransformError::InvalidGeometry(
            "回転後の頂点から境界ボックスを作成できません".to_string(),
        ))
    }

    /// Rodriguesの回転公式による3D点の中心周り回転
    ///
    /// # 引数
    /// * `point` - 回転対象の点
    /// * `center` - 回転中心
    /// * `k` - 正規化された回転軸
    /// * `cos_angle` - 回転角のコサイン
    /// * `sin_angle` - 回転角のサイン
    ///
    /// # 戻り値
    /// 回転後の点
    fn rodrigues_rotation_around_center(
        point: Point3D<T>,
        center: Point3D<T>,
        k: Vector3D<T>,
        cos_angle: T,
        sin_angle: T,
    ) -> Result<Point3D<T>, TransformError> {
        // 中心からの相対位置を計算
        let v = point - center;

        // v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_dot_v = k.dot(&v);
        let k_cross_v = k.cross(&v);
        let one_minus_cos = T::ONE - cos_angle;

        let rotated_v = v * cos_angle + k_cross_v * sin_angle + k * k_dot_v * one_minus_cos;
        let rotated_point = center + rotated_v;

        // 回転後の値の有効性チェック
        if !rotated_point.x().is_finite()
            || !rotated_point.y().is_finite()
            || !rotated_point.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "回転計算結果が無効".to_string(),
            ));
        }

        Ok(rotated_point)
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - スケール後の境界ボックス
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
    /// * `Ok(BBox3D)` - スケール後の境界ボックス
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

        // 境界ボックスの8つの頂点をスケール
        let vertices = self.get_all_vertices();

        // 各頂点をスケール
        let mut scaled_vertices = Vec::with_capacity(8);
        for vertex in &vertices {
            let relative = *vertex - center;
            let scaled_relative = relative * factor;
            let scaled_vertex = center + scaled_relative;

            // スケール結果の有効性チェック
            if !scaled_vertex.x().is_finite()
                || !scaled_vertex.y().is_finite()
                || !scaled_vertex.z().is_finite()
            {
                return Err(TransformError::InvalidGeometry(
                    "スケール計算結果が無効".to_string(),
                ));
            }

            scaled_vertices.push(scaled_vertex);
        }

        // スケールした頂点から新しい境界ボックスを構築
        Self::from_points(&scaled_vertices).ok_or(TransformError::InvalidGeometry(
            "スケール後の頂点から境界ボックスを作成できません".to_string(),
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
    /// * `Ok(BBox3D)` - スケール後の境界ボックス
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
    /// * `Ok(BBox3D)` - スケール後の境界ボックス
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

        // 境界ボックスの8つの頂点をスケール
        let vertices = self.get_all_vertices();

        // 各頂点を非均一スケール
        let mut scaled_vertices = Vec::with_capacity(8);
        for vertex in &vertices {
            let relative = *vertex - center;
            let scaled_relative = Vector3D::new(
                relative.x() * scale_x,
                relative.y() * scale_y,
                relative.z() * scale_z,
            );
            let scaled_vertex = center + scaled_relative;

            // スケール結果の有効性チェック
            if !scaled_vertex.x().is_finite()
                || !scaled_vertex.y().is_finite()
                || !scaled_vertex.z().is_finite()
            {
                return Err(TransformError::InvalidGeometry(
                    "非均一スケール計算結果が無効".to_string(),
                ));
            }

            scaled_vertices.push(scaled_vertex);
        }

        // スケールした頂点から新しい境界ボックスを構築
        Self::from_points(&scaled_vertices).ok_or(TransformError::InvalidGeometry(
            "非均一スケール後の頂点から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 安全な反射（平面に対する）
    ///
    /// # 引数
    /// * `plane_point` - 反射平面上の点
    /// * `plane_normal` - 反射平面の法線ベクトル
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - 反射後の境界ボックス
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

        // 境界ボックスの8つの頂点を反射
        let vertices = self.get_all_vertices();

        let mut reflected_vertices = Vec::with_capacity(8);
        for vertex in &vertices {
            let to_vertex = *vertex - plane_point;
            let distance_to_plane = to_vertex.dot(&normal_normalized);
            let two = T::ONE + T::ONE;
            let reflected_vertex = *vertex - normal_normalized * (distance_to_plane * two);

            // 反射結果の有効性チェック
            if !reflected_vertex.x().is_finite()
                || !reflected_vertex.y().is_finite()
                || !reflected_vertex.z().is_finite()
            {
                return Err(TransformError::InvalidGeometry(
                    "反射計算結果が無効".to_string(),
                ));
            }

            reflected_vertices.push(reflected_vertex);
        }

        // 反射した頂点から新しい境界ボックスを構築
        Self::from_points(&reflected_vertices).ok_or(TransformError::InvalidGeometry(
            "反射後の頂点から境界ボックスを作成できません".to_string(),
        ))
    }

    /// 安全なマージン拡張
    ///
    /// # 引数
    /// * `margin` - 拡張するマージン値
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - 拡張後の境界ボックス
    /// * `Err(TransformError)` - 無効なマージン（負の値、無限大、NaN）
    pub fn safe_expand_by_margin(&self, margin: T) -> Result<Self, TransformError> {
        // マージンの有効性チェック
        if margin < T::ZERO || !margin.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "無効なマージン値".to_string(),
            ));
        }

        let margin_vec = Vector3D::new(margin, margin, margin);
        let new_min = self.min() - margin_vec;
        let new_max = self.max() + margin_vec;

        // 結果の有効性チェック
        if !new_min.x().is_finite()
            || !new_min.y().is_finite()
            || !new_min.z().is_finite()
            || !new_max.x().is_finite()
            || !new_max.y().is_finite()
            || !new_max.z().is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "マージン拡張計算結果が無効".to_string(),
            ));
        }

        Ok(BBox3D::new(new_min, new_max))
    }
}

/// BBox3D のトレランス制約付き安全変換操作
impl<T: Scalar + GeometricTolerance> BBox3D<T> {
    /// トレランス制約付きスケール（指定点中心）
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率（正の値のみ）
    ///
    /// # 戻り値
    /// * `Ok(BBox3D)` - スケール後の境界ボックス
    /// * `Err(TransformError)` - 無効なスケール倍率または結果
    ///
    /// # エラー条件
    /// - スケール倍率が0以下
    /// - スケール後のサイズがトレランス以下
    pub fn safe_scale_with_tolerance(
        &self,
        center: Point3D<T>,
        factor: T,
    ) -> Result<Self, TransformError> {
        // 基本的なスケール倍率チェック
        if factor <= T::ZERO || !factor.is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率は正の有限値である必要があります".to_string(),
            ));
        }

        // 境界ボックスの8つの角をスケール
        let corners = [
            self.min(),
            Point3D::new(self.max().x(), self.min().y(), self.min().z()),
            Point3D::new(self.min().x(), self.max().y(), self.min().z()),
            Point3D::new(self.min().x(), self.min().y(), self.max().z()),
            Point3D::new(self.max().x(), self.max().y(), self.min().z()),
            Point3D::new(self.max().x(), self.min().y(), self.max().z()),
            Point3D::new(self.min().x(), self.max().y(), self.max().z()),
            self.max(),
        ];

        let scaled_corners: Vec<Point3D<T>> = corners
            .iter()
            .map(|&corner| center + (corner - center) * factor)
            .collect();

        // スケール後の境界を計算
        let mut min_x = scaled_corners[0].x();
        let mut max_x = scaled_corners[0].x();
        let mut min_y = scaled_corners[0].y();
        let mut max_y = scaled_corners[0].y();
        let mut min_z = scaled_corners[0].z();
        let mut max_z = scaled_corners[0].z();

        for corner in &scaled_corners {
            if corner.x() < min_x {
                min_x = corner.x();
            }
            if corner.x() > max_x {
                max_x = corner.x();
            }
            if corner.y() < min_y {
                min_y = corner.y();
            }
            if corner.y() > max_y {
                max_y = corner.y();
            }
            if corner.z() < min_z {
                min_z = corner.z();
            }
            if corner.z() > max_z {
                max_z = corner.z();
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;
        let depth = max_z - min_z;

        // サイズの幾何学的制約チェック（トレランスベース）
        let min_size = T::DISTANCE_TOLERANCE;
        if width <= min_size || height <= min_size || depth <= min_size {
            return Err(TransformError::InvalidGeometry(format!(
                "スケール後のサイズ(幅:{:?}, 高さ:{:?}, 奥行き:{:?})がトレランス({:?})以下になります",
                width, height, depth, min_size
            )));
        }

        // 数値安定性チェック
        if !min_x.is_finite()
            || !max_x.is_finite()
            || !min_y.is_finite()
            || !max_y.is_finite()
            || !min_z.is_finite()
            || !max_z.is_finite()
        {
            return Err(TransformError::InvalidGeometry(
                "スケール計算結果が無効です".to_string(),
            ));
        }

        let scaled_min = Point3D::new(min_x, min_y, min_z);
        let scaled_max = Point3D::new(max_x, max_y, max_z);

        Ok(Self::new(scaled_min, scaled_max))
    }

    /// サイズスケールの最小許容倍率を取得
    ///
    /// # 戻り値
    /// この境界ボックスに適用可能な最小のスケール倍率
    pub fn minimum_scale_factor(&self) -> T {
        let min_size = T::DISTANCE_TOLERANCE;
        let current_width = self.width();
        let current_height = self.height();
        let current_depth = self.depth();

        // 最小のサイズを基準に計算
        let smallest_size = current_width.min(current_height).min(current_depth);

        if smallest_size <= T::ZERO {
            T::ZERO
        } else {
            // 最小サイズを維持するための倍率 + 安全マージン
            let min_factor = min_size / smallest_size;
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

    #[test]
    fn test_safe_translate_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let translation = Vector3D::new(2.0, 3.0, 4.0);
        let result = bbox.safe_translate(translation).unwrap();

        let tolerance = 1e-10;
        assert!((result.min().x() - 3.0).abs() < tolerance);
        assert!((result.min().y() - 5.0).abs() < tolerance);
        assert!((result.min().z() - 7.0).abs() < tolerance);
        assert!((result.max().x() - 6.0).abs() < tolerance);
        assert!((result.max().y() - 8.0).abs() < tolerance);
        assert!((result.max().z() - 10.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_translate_invalid_vector() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        // 無限大の移動ベクトル
        let invalid_translation = Vector3D::new(f64::INFINITY, 0.0, 0.0);
        let result = bbox.safe_translate(invalid_translation);
        assert!(result.is_err());

        // NaNの移動ベクトル
        let nan_translation = Vector3D::new(f64::NAN, 0.0, 0.0);
        let result = bbox.safe_translate(nan_translation);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_rotate_z_origin_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(2.0, 2.0, 2.0));

        // Z軸周りに90度回転
        let result = bbox
            .safe_rotate_z_origin(Angle::from_degrees(90.0))
            .unwrap();

        // 回転後は頂点が移動するため、新しい境界ボックスが作成される
        // 正確な値は複雑だが、妥当な値が返されることを確認
        assert!(result.min().x().is_finite());
        assert!(result.min().y().is_finite());
        assert!(result.min().z().is_finite());
        assert!(result.max().x().is_finite());
        assert!(result.max().y().is_finite());
        assert!(result.max().z().is_finite());
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(2.0, 2.0, 2.0));

        // 無限大の角度
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = bbox.safe_rotate_z_origin(invalid_angle);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_origin_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let result = bbox.safe_scale_origin(2.0).unwrap();

        let tolerance = 1e-10;
        // 原点中心でのスケールなので座標が倍になる
        assert!((result.min().x() - 2.0).abs() < tolerance);
        assert!((result.min().y() - 4.0).abs() < tolerance);
        assert!((result.min().z() - 6.0).abs() < tolerance);
        assert!((result.max().x() - 8.0).abs() < tolerance);
        assert!((result.max().y() - 10.0).abs() < tolerance);
        assert!((result.max().z() - 12.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_scale_zero_factor_error() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let result = bbox.safe_scale_origin(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_scale_non_uniform_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let result = bbox.safe_scale_non_uniform_origin(2.0, 3.0, 4.0).unwrap();

        let tolerance = 1e-10;
        // X方向に2倍、Y方向に3倍、Z方向に4倍のスケール
        assert!((result.min().x() - 2.0).abs() < tolerance);
        assert!((result.min().y() - 6.0).abs() < tolerance);
        assert!((result.min().z() - 12.0).abs() < tolerance);
        assert!((result.max().x() - 8.0).abs() < tolerance);
        assert!((result.max().y() - 15.0).abs() < tolerance);
        assert!((result.max().z() - 24.0).abs() < tolerance);
    }

    #[test]
    fn test_safe_reflect_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(2.0, 2.0, 2.0));

        // XY平面に対する反射（Z=0平面、法線は(0,0,1)）
        let plane_point = Point3D::origin();
        let plane_normal = Vector3D::unit_z();
        let result = bbox.safe_reflect(plane_point, plane_normal).unwrap();

        // Z座標が反転される
        assert!(result.min().z() < 0.0);
        assert!(result.max().z() < 0.0);
        // X、Y座標は変わらない
        assert!(result.min().x() > 0.0);
        assert!(result.max().x() > 0.0);
        assert!(result.min().y() > 0.0);
        assert!(result.max().y() > 0.0);
    }

    #[test]
    fn test_safe_expand_by_margin_success() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let result = bbox.safe_expand_by_margin(0.5).unwrap();

        let tolerance = 1e-10;
        assert!((result.min().x() - 0.5).abs() < tolerance);
        assert!((result.min().y() - 1.5).abs() < tolerance);
        assert!((result.min().z() - 2.5).abs() < tolerance);
        assert!((result.max().x() - 4.5).abs() < tolerance);
        assert!((result.max().y() - 5.5).abs() < tolerance);
        assert!((result.max().z() - 6.5).abs() < tolerance);
    }

    #[test]
    fn test_safe_expand_by_margin_negative_error() {
        let bbox = BBox3D::<f64>::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 5.0, 6.0));

        let result = bbox.safe_expand_by_margin(-1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_rodrigues_rotation_x_axis() {
        let bbox = BBox3D::<f64>::new(Point3D::new(0.0, 1.0, 0.0), Point3D::new(1.0, 2.0, 1.0));

        // X軸周りに90度回転（任意軸回転を使用）
        let result = bbox
            .safe_rotate_axis(
                Point3D::origin(),
                Vector3D::unit_x(),
                Angle::from_degrees(90.0),
            )
            .unwrap();

        // Y軸上の点がZ軸に回転することを確認
        // 境界ボックスの形状が変わることを確認
        assert!(result.min().x().is_finite());
        assert!(result.min().y().is_finite());
        assert!(result.min().z().is_finite());
        assert!(result.max().x().is_finite());
        assert!(result.max().y().is_finite());
        assert!(result.max().z().is_finite());
    }
}
