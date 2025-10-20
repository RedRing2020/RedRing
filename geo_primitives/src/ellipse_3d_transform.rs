//! Ellipse3D の変換機能（基本実装）
//!
//! BasicTransform3D トレイトの実装のみ
//! Direction3D と Vector3D の正しい API に基づく
//! 内部的に安全なTransform実装を使用

use crate::{
    ellipse_3d::Ellipse3D, 
    point_3d::Point3D, 
    vector_3d::Vector3D, 
    transform_error::TransformError,
    Angle, Scalar
};
use geo_foundation::extensions::BasicTransform3D;

// ============================================================================
// BasicTransform3D Implementation
// ============================================================================

impl<T: Scalar> BasicTransform3D<T> for Ellipse3D<T> {
    type Point3D = Point3D<T>;
    type Vector3D = Vector3D<T>;
    type Transformed = Self;
    type Rotation3D = (Vector3D<T>, Angle<T>); // (軸, 角度) のタプル

    /// 平行移動
    ///
    /// 楕円を指定ベクトル分だけ平行移動
    ///
    /// # 引数
    /// * `translation` - 平行移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい楕円
    fn translate_3d(&self, translation: Self::Vector3D) -> Self::Transformed {
        // 安全な実装を使用してエラーハンドリング
        match self.safe_translate(translation) {
            Ok(result) => result,
            Err(_) => {
                // 平行移動は通常失敗しないが、万が一の場合は元の楕円を返す
                *self
            }
        }
    }

    /// 回転変換
    ///
    /// 指定点周りで楕円を回転
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `rotation` - 回転軸と角度のタプル
    ///
    /// # 戻り値
    /// 回転された新しい楕円（簡易実装）
    fn rotate_3d(&self, center: Self::Point3D, rotation: Self::Rotation3D) -> Self::Transformed {
        let (axis, angle) = rotation;
        // 安全な実装を使用してエラーハンドリング
        match self.safe_rotate(center, axis, angle) {
            Ok(result) => result,
            Err(_) => {
                // 回転が失敗した場合は元の楕円を返す
                // （例：ゼロベクトル軸、無効な楕円など）
                *self
            }
        }
    }

    /// 等方スケール
    ///
    /// 指定点を中心に楕円を等倍スケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい楕円
    fn scale_3d(&self, center: Self::Point3D, factor: T) -> Self::Transformed {
        // 安全な実装を使用してエラーハンドリング
        match self.safe_scale(center, factor) {
            Ok(result) => result,
            Err(_) => {
                // スケールが失敗した場合は元の楕円を返す
                // （例：ゼロスケール、無効な楕円など）
                *self
            }
        }
    }
}

// ============================================================================
// Additional Transform Methods (非トレイト)
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    /// 楕円の向きを反転
    ///
    /// 法線ベクトルの向きを反転
    ///
    /// # 戻り値
    /// 法線が反転された新しい楕円
    pub fn reverse(&self) -> Self {
        // 法線の反転は基本的に失敗しないが、安全のため確認
        match Self::new(
            self.center(),
            self.semi_major_axis(),
            self.semi_minor_axis(),
            -self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        ) {
            Some(ellipse) => ellipse,
            None => *self, // 万が一失敗した場合は元の楕円を返す
        }
    }

    /// 複合変換：平行移動 + 回転
    pub fn translate_and_rotate(
        &self,
        translation: Vector3D<T>,
        rotation_center: Point3D<T>,
        rotation_axis: Vector3D<T>,
        rotation_angle: Angle<T>,
    ) -> Self {
        let translated = self.translate_3d(translation);
        translated.rotate_3d(rotation_center, (rotation_axis, rotation_angle))
    }

    /// 複合変換：スケール + 平行移動
    pub fn scale_and_translate(
        &self,
        scale_center: Point3D<T>,
        scale_factor: T,
        translation: Vector3D<T>,
    ) -> Self {
        let scaled = self.scale_3d(scale_center, scale_factor);
        scaled.translate_3d(translation)
    }

    /// 変換後の楕円の妥当性チェック
    pub fn is_valid_transform(&self) -> bool {
        // 半軸長の妥当性
        if self.semi_major_axis() <= T::ZERO || self.semi_minor_axis() <= T::ZERO {
            return false;
        }

        // 長軸 >= 短軸
        if self.semi_major_axis() < self.semi_minor_axis() {
            return false;
        }

        true
    }

    /// 変換の等価性チェック（テスト用）
    pub fn transform_equivalent(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の等価性
        if self.center().distance_to(&other.center()) > tolerance {
            return false;
        }

        // 半軸長の等価性
        let major_diff = (self.semi_major_axis() - other.semi_major_axis()).abs();
        let minor_diff = (self.semi_minor_axis() - other.semi_minor_axis()).abs();

        if major_diff > tolerance || minor_diff > tolerance {
            return false;
        }

        // 法線方向の等価性（方向ベクトルのドット積）
        let normal_dot = self.normal().dot(&other.normal()).abs();
        normal_dot > T::ONE - tolerance
    }
}
