//! Vector2D変換機能の拡張実装
//!
//! geo_foundation統合による統一Transform Foundation システム
//! BasicTransform + 高度変換操作の実装

use crate::{Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// Transform Traits（変換操作のためのトレイト）
// ============================================================================

/// 2Dベクトルを方向ベクトルとして変換するトレイト
pub trait TransformVector2D<T: Scalar> {
    /// ベクトルを方向ベクトルとして変換（平行移動なし）
    fn transform_vector_2d(&self, vector: &Vector2D<T>) -> Vector2D<T>;
}

/// 2D点として変換するトレイト
pub trait TransformPoint2D<T: Scalar> {
    /// 点を変換（平行移動あり）
    fn transform_point_2d(&self, point: &Point2D<T>) -> Point2D<T>;
}

// ============================================================================
// Vector2D Transform Extensions
// ============================================================================

impl<T: Scalar> Vector2D<T> {
    // ========================================================================
    // Advanced Transform Methods
    // ========================================================================

    /// 3x3変換行列でベクトルを変換（方向ベクトルとして）
    ///
    /// ベクトルを(x, y, 0)として扱い、回転・スケールのみ適用
    /// 平行移動は無視される（方向ベクトルの特性）
    ///
    /// # 引数
    /// * `matrix` - 3x3変換行列の参照
    ///
    /// # 戻り値
    /// 変換された新しいベクトル
    ///
    /// # 例
    /// ```
    /// use geo_primitives::Vector2D;
    /// let v = Vector2D::new(1.0, 0.0);
    /// // matrix は 3x3 変換行列
    /// // let transformed = v.transform_vector(&matrix);
    /// ```
    pub fn transform_vector<M: TransformVector2D<T>>(&self, matrix: &M) -> Vector2D<T> {
        matrix.transform_vector_2d(self)
    }

    /// ベクトルの終点を点として変換
    ///
    /// ベクトルを原点からの位置ベクトルと見なして、
    /// 平行移動も含む完全な変換を適用
    ///
    /// # 引数
    /// * `matrix` - 3x3変換行列の参照
    ///
    /// # 戻り値
    /// 変換された点
    ///
    /// # 例
    /// ```
    /// use geo_primitives::Vector2D;
    /// let v = Vector2D::new(1.0, 1.0);
    /// // matrix は 3x3 変換行列
    /// // let transformed_point = v.transform_point(&matrix);
    /// ```
    pub fn transform_point<M: TransformPoint2D<T>>(&self, matrix: &M) -> Point2D<T> {
        let point = Point2D::new(self.x(), self.y());
        matrix.transform_point_2d(&point)
    }

    // ========================================================================
    // Specialized Rotation Methods（軸指定回転）
    // ========================================================================

    /// Z軸周りの回転（2D回転）
    ///
    /// 2D平面内でベクトルを原点周りに回転させる
    ///
    /// # 引数
    /// * `angle_rad` - 回転角度（ラジアン、反時計回り）
    ///
    /// # 戻り値
    /// 回転後の新しいベクトル
    ///
    /// # 例
    /// ```
    /// use geo_primitives::Vector2D;
    /// use std::f64::consts::PI;
    ///
    /// let v = Vector2D::new(1.0, 0.0);
    /// let rotated = v.rotate_z(PI / 2.0); // 90度回転
    /// // rotated ≈ Vector2D::new(0.0, 1.0)
    /// ```
    pub fn rotate_z(&self, angle_rad: T) -> Vector2D<T>
    where
        T: Into<f64> + From<f64>,
    {
        let angle: f64 = angle_rad.into();
        let cos_a = T::from(angle.cos());
        let sin_a = T::from(angle.sin());

        Vector2D::new(
            cos_a * self.x() - sin_a * self.y(),
            sin_a * self.x() + cos_a * self.y(),
        )
    }
}

// ============================================================================
// BasicTransform Implementation
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Vector2D<T> {
    type Transformed = Vector2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// ベクトルによる平行移動
    ///
    /// ベクトル同士の加算として実装
    ///
    /// # 引数
    /// * `translation` - 移動量ベクトル
    ///
    /// # 戻り値
    /// 平行移動後の新しいベクトル
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        *self + translation
    }

    /// 指定した点を中心とした回転
    ///
    /// ベクトルを点として扱い、中心点周りに回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転後の新しいベクトル
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // ベクトルを点として扱って回転
        let point = Point2D::new(self.x(), self.y());
        let rotated_point = point.translate(Vector2D::new(-center.x(), -center.y()));
        let rotated = rotated_point.rotate_radians(angle.to_radians());
        let final_point =
            Point2D::new(rotated.x(), rotated.y()).translate(Vector2D::new(center.x(), center.y()));
        Vector2D::new(final_point.x(), final_point.y())
    }

    /// 指定した点を中心としたスケール変更
    ///
    /// ベクトルを点として扱い、中心点からの距離をスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール係数
    ///
    /// # 戻り値
    /// スケール後の新しいベクトル
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // ベクトルを点として扱ってスケール
        let point = Point2D::new(self.x(), self.y());
        let offset = Vector2D::new(point.x() - center.x(), point.y() - center.y());
        let scaled_offset = Vector2D::new(offset.x() * factor, offset.y() * factor);
        Vector2D::new(
            center.x() + scaled_offset.x(),
            center.y() + scaled_offset.y(),
        )
    }
}
