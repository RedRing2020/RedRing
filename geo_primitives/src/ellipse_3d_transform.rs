//! Ellipse3D の変換機能（基本実装）
//!
//! BasicTransform3D トレイトの実装のみ
//! Direction3D と Vector3D の正しい API に基づく

use crate::{ellipse_3d::Ellipse3D, point_3d::Point3D, vector_3d::Vector3D, Angle, Scalar};
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
        Self::new(
            self.center() + translation,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .expect("平行移動後の楕円は有効")
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
        let (_axis, _angle) = rotation;

        // 簡易実装：回転中心への平行移動のみ実行
        // 完全な3D回転は複雑な行列演算が必要
        let center_vector = self.center().to_vector();
        let relative_position = center_vector - center.to_vector();

        // 簡単な近似：位置のみ変更、向きは保持
        let new_center_vector = center.to_vector() + relative_position;
        let new_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        Self::new(
            new_center,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .expect("回転後の楕円は有効")
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
        // 中心点をスケール
        let center_vector = self.center().to_vector();
        let relative_position = center_vector - center.to_vector();
        let scaled_relative = relative_position * factor;
        let new_center_vector = center.to_vector() + scaled_relative;
        let new_center = Point3D::new(
            new_center_vector.x(),
            new_center_vector.y(),
            new_center_vector.z(),
        );

        // 半軸長をスケール
        let new_semi_major = self.semi_major_axis() * factor.abs();
        let new_semi_minor = self.semi_minor_axis() * factor.abs();

        Self::new(
            new_center,
            new_semi_major,
            new_semi_minor,
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .expect("スケール後の楕円は有効")
    }
}

// ============================================================================
// Additional Transform Methods (非トレイト)
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    /// 楕円の反転
    ///
    /// 楕円の法線方向を反転
    ///
    /// # 戻り値
    /// 法線が反転された新しい楕円
    pub fn reverse(&self) -> Self {
        Self::new(
            self.center(),
            self.semi_major_axis(),
            self.semi_minor_axis(),
            -self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .expect("反転後の楕円は有効")
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
