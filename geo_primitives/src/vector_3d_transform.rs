//! Vector3D変換機能の拡張実装
//!
//! geo_foundation統合による統一Transform Foundation システム
//! BasicTransform + 高度変換操作の実装

use crate::{Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// Transform Traits（変換操作のためのトレイト）
// ============================================================================

/// 3Dベクトルを方向ベクトルとして変換するトレイト
pub trait TransformVector3D<T: Scalar> {
    /// ベクトルを方向ベクトルとして変換（平行移動なし）
    fn transform_vector_3d(&self, vector: &Vector3D<T>) -> Vector3D<T>;
}

/// 3D点として変換するトレイト
pub trait TransformPoint3D<T: Scalar> {
    /// 点を変換（平行移動あり）
    fn transform_point_3d(&self, point: &Point3D<T>) -> Point3D<T>;
}

// ============================================================================
// Vector3D Transform Extensions
// ============================================================================

impl<T: Scalar> Vector3D<T> {
    // ========================================================================
    // Advanced Transform Methods
    // ========================================================================

    /// 4x4変換行列でベクトルを変換（方向ベクトルとして）
    ///
    /// ベクトルを(x, y, z, 0)として扱い、回転・スケールのみ適用
    /// 平行移動は無視される（方向ベクトルの特性）
    ///
    /// # 引数
    /// * `matrix` - 4x4変換行列の参照
    ///
    /// # 戻り値
    /// 変換された新しいベクトル
    ///
    /// # 例
    /// ```
    /// use geo_primitives::Vector3D;
    /// let v = Vector3D::new(1.0, 0.0, 0.0);
    /// // matrix は 4x4 変換行列
    /// // let transformed = v.transform_vector(&matrix);
    /// ```
    pub fn transform_vector<M>(&self, matrix: &M) -> Self
    where
        M: TransformVector3D<T>,
    {
        matrix.transform_vector_3d(self)
    }

    /// 4x4変換行列でベクトルを点として変換
    ///
    /// ベクトルを(x, y, z, 1)として扱い、平行移動・回転・スケールを適用
    /// 原点からの位置ベクトルとして解釈
    ///
    /// # 引数
    /// * `matrix` - 4x4変換行列の参照
    ///
    /// # 戻り値
    /// 変換された新しいベクトル
    ///
    /// # 例
    /// ```
    /// use geo_primitives::Vector3D;
    /// let v = Vector3D::new(1.0, 2.0, 3.0);
    /// // matrix は 4x4 変換行列
    /// // let transformed = v.transform_point(&matrix);
    /// ```
    pub fn transform_point<M>(&self, matrix: &M) -> Self
    where
        M: TransformPoint3D<T>,
    {
        let point = self.to_point();
        let transformed_point = matrix.transform_point_3d(&point);
        Self::new(
            transformed_point.x(),
            transformed_point.y(),
            transformed_point.z(),
        )
    }

    // ========================================================================
    // Rotation Transform Methods
    // ========================================================================

    /// 回転変換のみを適用
    ///
    /// 指定された角度だけZ軸周りに回転
    ///
    /// # 引数
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # 戻り値
    /// 回転された新しいベクトル
    pub fn rotate_z(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
            self.z(),
        )
    }

    /// X軸周りの回転
    pub fn rotate_x(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
            self.x(),
            self.y() * cos_a - self.z() * sin_a,
            self.y() * sin_a + self.z() * cos_a,
        )
    }

    /// Y軸周りの回転
    pub fn rotate_y(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        Self::new(
            self.x() * cos_a + self.z() * sin_a,
            self.y(),
            -self.x() * sin_a + self.z() * cos_a,
        )
    }

    // ========================================================================
    // Composite Transform Methods
    // ========================================================================

    /// オイラー角による複合回転（ZYX順）
    pub fn rotate_euler_zyx(&self, yaw: T, pitch: T, roll: T) -> Self {
        self.rotate_z(yaw).rotate_y(pitch).rotate_x(roll)
    }

    /// オイラー角による複合回転（XYZ順）
    pub fn rotate_euler_xyz(&self, roll: T, pitch: T, yaw: T) -> Self {
        self.rotate_x(roll).rotate_y(pitch).rotate_z(yaw)
    }

    /// 任意軸周りの回転（ロドリゲスの公式）
    pub fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self {
        let normalized_axis = axis.normalize();
        if normalized_axis.is_zero() {
            return *self;
        }

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        let dot_product = self.dot(&normalized_axis);
        let cross_product = normalized_axis.cross(self);

        *self * cos_a + cross_product * sin_a + normalized_axis * (dot_product * one_minus_cos)
    }
}

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Vector3D<T> {
    type Transformed = Vector3D<T>;
    type Vector2D = Vector3D<T>; // 3Dベクトルを2D操作でも使用
    type Point2D = Point3D<T>; // 3D点を2D操作でも使用
    type Angle = Angle<T>;

    /// 平行移動（ベクトルの加算）
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        self.safe_translate(translation).unwrap()
    }

    /// Z軸周りの回転
    fn rotate(&self, _center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        self.safe_rotate_z_origin(angle).unwrap()
    }

    /// 等方スケール
    fn scale(&self, _center: Self::Point2D, factor: T) -> Self::Transformed {
        self.safe_scale_origin(factor).unwrap()
    }
}
