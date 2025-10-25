//! Direction2D Safe Transform Implementation
//!
//! Direction2Dの安全な変換操作を提供します。
//! エラーハンドリングを含む変換メソッドを実装。

use crate::{Direction2D, Point2D, Vector2D};
use geo_foundation::{prelude::Angle, Scalar, TransformError};

impl<T: Scalar> Direction2D<T> {
    // ========================================================================
    // Safe Transform Operations - Direction specific
    // ========================================================================

    /// 安全な正規化（既に正規化済みだが一貫性のため）
    ///
    /// # 引数
    /// - なし（既に正規化済み）
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 正規化された方向（変更なし）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_normalize(&self) -> Result<Self, TransformError> {
        // Directionは常に正規化済みなので、そのまま返す
        Ok(*self)
    }

    /// 安全な回転（点を中心とした回転）
    ///
    /// # 引数
    /// - `_center`: 回転中心（方向ベクトルなので実際は使用されない）
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な回転角度の場合
    pub fn safe_rotate(
        &self,
        _center: Point2D<T>,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 角度の妥当性チェック
        let angle_rad = angle.to_radians();
        if !angle_rad.is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度が無限大またはNaNです".to_string(),
            ));
        }

        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        let x = self.x();
        let y = self.y();

        let new_x = cos_angle * x - sin_angle * y;
        let new_y = sin_angle * x + cos_angle * y;

        let rotated_vector = Vector2D::new(new_x, new_y);

        // 方向ベクトルなので、結果も必ず正規化されているはず
        Self::from_vector(rotated_vector).ok_or_else(|| {
            TransformError::InvalidGeometry("回転後の方向ベクトルが無効になりました".to_string())
        })
    }

    /// 安全な回転（原点中心）
    ///
    /// # 引数
    /// - `angle`: 回転角度
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 回転後の方向
    /// - `Err(TransformError)` - 無効な回転角度の場合
    pub fn safe_rotate_origin(&self, angle: Angle<T>) -> Result<Self, TransformError> {
        self.safe_rotate(Point2D::origin(), angle)
    }

    /// 安全なスケール（方向ベクトルはスケールされない）
    ///
    /// # 引数
    /// - `_center`: スケール中心（方向ベクトルなので実際は使用されない）
    /// - `_factor`: スケール倍率（方向ベクトルなので実際は使用されない）
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 変更されない方向（単位ベクトルを保持）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_scale(&self, _center: Point2D<T>, _factor: T) -> Result<Self, TransformError> {
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
    /// - `Ok(Direction2D)` - 変更されない方向（単位ベクトルを保持）
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_scale_origin(&self, _factor: T) -> Result<Self, TransformError> {
        self.safe_scale(Point2D::origin(), _factor)
    }

    /// 安全な平行移動（方向ベクトルは移動されない）
    ///
    /// # 引数
    /// - `_translation`: 移動ベクトル（方向ベクトルなので実際は使用されない）
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 変更されない方向
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_translate(&self, _translation: Vector2D<T>) -> Result<Self, TransformError> {
        // 方向ベクトルは平行移動されない（位置を持たない）
        Ok(*self)
    }

    /// 安全な反転
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 反転された方向
    /// - `Err(TransformError)` - 理論上発生しないが、一貫性のため
    pub fn safe_reverse(&self) -> Result<Self, TransformError> {
        Ok(self.reverse())
    }

    // ========================================================================
    // Advanced Safe Transform Operations
    // ========================================================================

    /// 安全な反射（軸に対する反射）
    ///
    /// # 引数
    /// - `axis`: 反射軸の方向
    ///
    /// # 戻り値
    /// - `Ok(Direction2D)` - 反射後の方向
    /// - `Err(TransformError)` - 無効な軸の場合
    pub fn safe_reflect(&self, axis: &Direction2D<T>) -> Result<Self, TransformError> {
        // 反射公式: v' = v - 2(v·n)n (nは軸の法線)
        // 2D では軸の法線は軸を90度回転したもの
        let axis_normal = axis.rotate_90();
        let dot_product = self.dot(&axis_normal);
        let two = T::ONE + T::ONE;

        let reflected_vector = self.as_vector() - axis_normal.as_vector() * (two * dot_product);

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
    /// - `Ok(Direction2D)` - 直交成分の方向
    /// - `Err(TransformError)` - 平行ベクトルで直交成分が存在しない場合
    pub fn safe_orthogonalize(&self, other: &Direction2D<T>) -> Result<Self, TransformError> {
        let dot_product = self.dot(other);
        let orthogonal_vector = self.as_vector() - other.as_vector() * dot_product;

        Self::from_vector(orthogonal_vector).ok_or_else(|| {
            TransformError::InvalidGeometry(
                "直交化後のベクトルがゼロベクトルになりました（平行ベクトル）".to_string(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_normalize() {
        let dir = Direction2D::<f64>::positive_x();
        let result = dir.safe_normalize().unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_rotate_origin() {
        let dir = Direction2D::<f64>::positive_x();

        // 90度回転
        let result = dir.safe_rotate_origin(Angle::from_degrees(90.0)).unwrap();
        let expected: Direction2D<f64> = Direction2D::positive_y();

        let tolerance = 1e-10_f64;
        assert!((result.x() - expected.x()).abs() < tolerance);
        assert!((result.y() - expected.y()).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_with_center() {
        let dir = Direction2D::<f64>::positive_x();
        let center = Point2D::new(1.0, 1.0); // 中心は使われないが指定

        // 180度回転
        let result = dir.safe_rotate(center, Angle::from_degrees(180.0)).unwrap();
        let expected: Direction2D<f64> = Direction2D::negative_x();

        let tolerance = 1e-10_f64;
        assert!((result.x() - expected.x()).abs() < tolerance);
        assert!((result.y() - expected.y()).abs() < tolerance);
    }

    #[test]
    fn test_safe_rotate_invalid_angle() {
        let dir = Direction2D::<f64>::positive_x();

        // 無限大の角度でエラー
        let invalid_angle = Angle::from_radians(f64::INFINITY);
        let result = dir.safe_rotate_origin(invalid_angle);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidRotation(_)
        ));
    }

    #[test]
    fn test_safe_scale_no_change() {
        let dir = Direction2D::<f64>::positive_x();
        let center = Point2D::origin();

        // 方向ベクトルはスケールされない
        let result = dir.safe_scale(center, 5.0).unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_translate_no_change() {
        let dir = Direction2D::<f64>::positive_x();
        let translation = Vector2D::new(10.0, 20.0);

        // 方向ベクトルは平行移動されない
        let result = dir.safe_translate(translation).unwrap();
        assert_eq!(result, dir);
    }

    #[test]
    fn test_safe_reverse() {
        let dir = Direction2D::<f64>::positive_x();
        let result = dir.safe_reverse().unwrap();
        let expected = Direction2D::negative_x();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_safe_reflect() {
        let dir = Direction2D::<f64>::positive_x();
        let axis = Direction2D::positive_y(); // Y軸に対する反射

        let result = dir.safe_reflect(&axis).unwrap();
        let expected: Direction2D<f64> = Direction2D::negative_x();

        let tolerance = 1e-10_f64;
        assert!((result.x() - expected.x()).abs() < tolerance);
        assert!((result.y() - expected.y()).abs() < tolerance);
    }

    #[test]
    fn test_safe_orthogonalize() {
        // 45度の方向を基準に直交化
        let dir = Direction2D::<f64>::new(1.0, 1.0).unwrap(); // 45度
        let other = Direction2D::positive_x(); // X軸

        let result = dir.safe_orthogonalize(&other).unwrap();

        // 結果はY軸方向になるはず
        let tolerance = 1e-10_f64;
        assert!((result.x()).abs() < tolerance);
        assert!(result.y() > 0.0);
    }

    #[test]
    fn test_safe_orthogonalize_parallel_vectors() {
        let dir = Direction2D::<f64>::positive_x();
        let other = Direction2D::positive_x(); // 同じ方向

        // 平行ベクトルは直交化できない
        let result = dir.safe_orthogonalize(&other);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransformError::InvalidGeometry(_)
        ));
    }
}
