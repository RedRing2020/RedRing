//! Direction3D Safe Transform Implementation
//!
//! Direction3Dの安全な変換操作を提供します。
//! エラーハンドリングを含む変換メソッドを実装。

use crate::{Direction3D, Point3D, Vector3D};
use geo_foundation::{prelude::Angle, Scalar, TransformError};

impl<T: Scalar> Direction3D<T> {
    // ========================================================================
    // Safe Transform Operations - Direction specific
    // ========================================================================

    /// 安全な正規化（既に正規化済みだが一貫性のため）
    ///
    /// # 引数
    /// - なし（既に正規化済み）
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 正規化された方向（変更なし）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_normalize(&self) -> Result<Self, TransformError> {
        // Directionは常に正規化済みなので、そのまま返す
        Ok(*self)
    }

    /// 安全な平行移動（方向ベクトルは移動されない）
    ///
    /// # 引数
    /// - `_translation`: 移動ベクトル（方向ベクトルなので実際は使用されない）
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 変更されない方向
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_translate(&self, _translation: Vector3D<T>) -> Result<Self, TransformError> {
        // 方向ベクトルは平行移動されない（位置を持たない）
        Ok(*self)
    }

    /// 安全なスケール（方向ベクトルはスケールされない）
    ///
    /// # 引数
    /// - `_center`: スケール中心（方向ベクトルなので実際は使用されない）
    /// - `_factor`: スケール倍率（方向ベクトルなので実際は使用されない）
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 変更されない方向（単位ベクトルを保持）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_scale(&self, _center: Point3D<T>, _factor: T) -> Result<Self, TransformError> {
        // 方向ベクトルは単位ベクトルなので、スケールしても意味がない
        // 常に同じ方向を返す
        Ok(*self)
    }

    /// 安全なスケール（原点中心）
    ///
    /// # 引数
    /// - `_factor`: スケール倍率（方向ベクトルなので実際は使用されない）
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 変更されない方向（単位ベクトルを保持）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_scale_origin(&self, _factor: T) -> Result<Self, TransformError> {
        self.safe_scale(Point3D::origin(), _factor)
    }

    /// 安全な軸回転（任意軸周りの回転）
    ///
    /// # 引数
    /// - `center`: 回転中心（方向ベクトルなので実際は使用されない）
    /// - `axis`: 回転軸の方向
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な回転軸や角度の場合
    pub fn safe_rotate_axis(
        &self,
        _center: Point3D<T>,
        axis: &Direction3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の妥当性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度が無限大またはNaNです".to_string(),
            ));
        }

        // Rodrigues回転公式を使用
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let one_minus_cos = T::ONE - cos_angle;

        let k = axis.as_vector();
        let v = self.as_vector();

        // v_rot = v*cos(θ) + (k×v)*sin(θ) + k*(k·v)*(1-cos(θ))
        let k_dot_v = k.dot(&v);
        let k_cross_v = k.cross(&v);

        let rotated = v * cos_angle + k_cross_v * sin_angle + k * (k_dot_v * one_minus_cos);

        Self::from_vector(rotated).ok_or_else(|| {
            TransformError::InvalidGeometry("回転後の方向ベクトルが無効になりました".to_string())
        })
    }

    /// 安全な軸回転（原点中心）
    ///
    /// # 引数
    /// - `axis`: 回転軸の方向
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な回転軸や角度の場合
    pub fn safe_rotate_axis_origin(
        &self,
        axis: &Direction3D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        self.safe_rotate_axis(Point3D::origin(), axis, angle)
    }

    /// 安全なX軸回転（原点中心）
    ///
    /// # 引数
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な角度の場合
    pub fn safe_rotate_x_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_axis_origin(&Direction3D::positive_x(), angle)
    }

    /// 安全なY軸回転（原点中心）
    ///
    /// # 引数
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な角度の場合
    pub fn safe_rotate_y_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_axis_origin(&Direction3D::positive_y(), angle)
    }

    /// 安全なZ軸回転（原点中心）
    ///
    /// # 引数
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な角度の場合
    pub fn safe_rotate_z_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate_axis_origin(&Direction3D::positive_z(), angle)
    }

    /// 安全な反転
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 反転された方向
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_reverse(&self) -> Result<Self, TransformError> {
        Ok(self.reverse())
    }

    // ========================================================================
    // Advanced Safe Transform Operations
    // ========================================================================

    /// 安全な反射（平面に対する反射）
    ///
    /// # 引数
    /// - `normal`: 反射平面の法線ベクトル
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 反射後の方向
    /// - `Err(TransformError)` - 無効な法線の場合
    pub fn safe_reflect(&self, normal: &Direction3D<T>) -> Result<Self, TransformError> {
        // 反射公式: v' = v - 2(v·n)n
        let dot_product = self.dot(normal);
        let two = T::ONE + T::ONE;

        let reflected_vector = self.as_vector() - normal.as_vector() * (two * dot_product);

        Self::from_vector(reflected_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("反射後の方向ベクトルが無効になりました".to_string())
        })
    }

    /// 安全な直交化（他のベクトルに対して直交する成分を取得）
    ///
    /// # 引数
    /// - `other`: 基準となる方向
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 直交成分の方向
    /// - `Err(TransformError)` - 平行ベクトルで直交成分が存在しない場合
    pub fn safe_orthogonalize(&self, other: &Direction3D<T>) -> Result<Self, TransformError> {
        let dot_product = self.dot(other);
        let orthogonal_vector = self.as_vector() - other.as_vector() * dot_product;

        Self::from_vector(orthogonal_vector).ok_or_else(|| {
            TransformError::InvalidGeometry(
                "直交化後のベクトルがゼロベクトルになりました（平行ベクトル）".to_string(),
            )
        })
    }

    /// 安全な球面線形補間（SLERP）
    ///
    /// # 引数
    /// - `other`: 補間先の方向
    /// - `t`: 補間パラメータ（0.0～1.0）
    ///
    /// # 戻り値
    /// - `Ok(Direction3D)` - 補間された方向
    /// - `Err(TransformError)` - 無効なパラメータの場合
    pub fn safe_slerp(&self, other: &Direction3D<T>, t: T) -> Result<Self, TransformError> {
        // パラメータの妥当性チェック
        if t < T::ZERO || t > T::ONE {
            return Err(TransformError::InvalidRotation(format!(
                "SLERP パラメータは0.0～1.0の範囲である必要があります: {}",
                t
            )));
        }

        if !t.is_finite() {
            return Err(TransformError::InvalidRotation(
                "SLERP パラメータが無限大またはNaNです".to_string(),
            ));
        }

        let dot_product = self.dot(other);

        // ほぼ同じ方向の場合は線形補間
        let threshold = T::ONE - T::EPSILON * (T::ONE + T::ONE + T::ONE + T::ONE); // 1.0 - 4*EPSILON
        if dot_product > threshold {
            let lerp_vector = self.as_vector() * (T::ONE - t) + other.as_vector() * t;
            return Self::from_vector(lerp_vector).ok_or_else(|| {
                TransformError::InvalidGeometry("SLERP結果が無効になりました".to_string())
            });
        }

        // 球面線形補間
        let angle = dot_product.acos();
        let sin_angle = angle.sin();

        if sin_angle.abs() < T::EPSILON {
            return Err(TransformError::InvalidGeometry(
                "SLERP: 補間角度のsinが0に近すぎます".to_string(),
            ));
        }

        let sin_t_angle = (t * angle).sin();
        let sin_1_minus_t_angle = ((T::ONE - t) * angle).sin();

        let slerp_vector =
            (self.as_vector() * sin_1_minus_t_angle + other.as_vector() * sin_t_angle) / sin_angle;

        Self::from_vector(slerp_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("SLERP結果が無効になりました".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_normalize() {
        let dir = Direction3D::<f64>::positive_x();
        let result = dir.safe_normalize().unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_translate_no_change() {
        let dir = Direction3D::<f64>::positive_x();
        let translation = Vector3D::new(10.0, 20.0, 30.0);

        // 方向ベクトルは平行移動されない
        let result = dir.safe_translate(translation).unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_scale_no_change() {
        let dir = Direction3D::<f64>::positive_x();
        let center = Point3D::origin();

        // 方向ベクトルはスケールされない
        let result = dir.safe_scale(center, 5.0).unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_rotate_z_origin() {
        let dir = Direction3D::<f64>::positive_x();

        // Z軸周りに90度回転（X軸 → Y軸）
        let result = dir.safe_rotate_z_origin(Angle::from_degrees(90.0)).unwrap();
        let expected: Direction3D<f64> = Direction3D::positive_y();

        let tolerance = 1e-10_f64;
        assert!((result.x() as f64 - expected.x() as f64).abs() < tolerance);
        assert!((result.y() as f64 - expected.y() as f64).abs() < tolerance);
        assert!((result.z() as f64 - expected.z() as f64).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_x_origin() {
        let dir = Direction3D::<f64>::positive_y();

        // X軸周りに90度回転（Y軸 → Z軸）
        let result = dir.safe_rotate_x_origin(Angle::from_degrees(90.0)).unwrap();
        let expected: Direction3D<f64> = Direction3D::positive_z();

        let tolerance = 1e-10_f64;
        assert!((result.x() as f64 - expected.x() as f64).abs() < tolerance);
        assert!((result.y() as f64 - expected.y() as f64).abs() < tolerance);
        assert!((result.z() as f64 - expected.z() as f64).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_y_origin() {
        let dir = Direction3D::<f64>::positive_z();

        // Y軸周りに90度回転（Z軸 → X軸）
        let result = dir.safe_rotate_y_origin(Angle::from_degrees(90.0)).unwrap();
        let expected: Direction3D<f64> = Direction3D::positive_x();

        let tolerance = 1e-10_f64;
        assert!((result.x() as f64 - expected.x() as f64).abs() < tolerance);
        assert!((result.y() as f64 - expected.y() as f64).abs() < tolerance);
        assert!((result.z() as f64 - expected.z() as f64).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_axis_origin() {
        let dir = Direction3D::<f64>::positive_x();
        let axis = Direction3D::positive_z();

        // Z軸周りに180度回転
        let result = dir
            .safe_rotate_axis_origin(&axis, Angle::from_degrees(180.0))
            .unwrap();
        let expected: Direction3D<f64> = Direction3D::negative_x();

        let tolerance = 1e-10_f64;
        assert!((result.x() as f64 - expected.x() as f64).abs() < tolerance);
        assert!((result.y() as f64 - expected.y() as f64).abs() < tolerance);
        assert!((result.z() as f64 - expected.z() as f64).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let dir = Direction3D::<f64>::positive_x();
        let axis = Direction3D::positive_z();

        // 無限大の角度でエラー
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = dir.safe_rotate_axis_origin(&axis, invalid_angle);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));
    }

    #[test]
    fn test_safe_reverse() {
        let dir = Direction3D::<f64>::positive_x();
        let result = dir.safe_reverse().unwrap();
        let expected = Direction3D::negative_x();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_safe_reflect() {
        let dir = Direction3D::<f64>::positive_x();
        let normal = Direction3D::positive_y(); // YZ平面に対する反射

        let result = dir.safe_reflect(&normal).unwrap();
        let expected: Direction3D<f64> = Direction3D::positive_x(); // X成分は変わらない

        let tolerance = 1e-10_f64;
        assert!((result.x() as f64 - expected.x() as f64).abs() < tolerance);
        assert!((result.y() as f64 - expected.y() as f64).abs() < tolerance);
        assert!((result.z() as f64 - expected.z() as f64).abs() < tolerance);
    }

    #[test]
    fn test_safe_orthogonalize() {
        // 対角方向を基準に直交化
        let dir = Direction3D::<f64>::new(1.0, 1.0, 0.0).unwrap();
        let other = Direction3D::positive_x();

        let result = dir.safe_orthogonalize(&other).unwrap();

        // 結果とotherは直交するはず
        assert!((result.dot(&other)).abs() < 1e-10);
    }

    #[test]
    fn test_safe_orthogonalize_parallel_vectors() {
        let dir = Direction3D::<f64>::positive_x();
        let other = Direction3D::positive_x(); // 同じ方向

        // 平行ベクトルは直交化できない
        let result = dir.safe_orthogonalize(&other);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidGeometry(_)
        ));
    }

    #[test]
    fn test_safe_slerp() {
        let dir1 = Direction3D::<f64>::positive_x();
        let dir2 = Direction3D::<f64>::positive_y();

        // 中点での補間
        let result = dir1.safe_slerp(&dir2, 0.5).unwrap();

        // 結果は対角方向になるはず
        let tolerance = 1e-10_f64;
        assert!((result.x() - result.y()).abs() < tolerance);
        assert!(result.x() > 0.0 && result.y() > 0.0);
        assert!((result.z()).abs() < tolerance);
    }

    #[test]
    fn test_safe_slerp_invalid_parameter() {
        let dir1 = Direction3D::<f64>::positive_x();
        let dir2 = Direction3D::<f64>::positive_y();

        // 範囲外のパラメータでエラー
        let result = dir1.safe_slerp(&dir2, -0.1);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));

        let result = dir1.safe_slerp(&dir2, 1.1);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));
    }

    #[test]
    fn test_safe_slerp_same_direction() {
        let dir = Direction3D::<f64>::positive_x();

        // 同じ方向での補間
        let result = dir.safe_slerp(&dir, 0.5).unwrap();
        assert_eq!(result, dir);
    }
}
