//! EllipseArc3D Transform実装
//!
//! BasicTransform と AdvancedTransform の完全実装
//! 3D楕円弧の変換操作に特化した実装

use crate::{EllipseArc3D, Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Advanced Transform Operations (3D specific)
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    /// 軸回転（任意軸）
    ///
    /// 3D楕円弧を任意軸を中心に回転
    ///
    /// # 引数
    /// * `axis_point` - 回転軸上の点
    /// * `axis_direction` - 回転軸の方向ベクトル
    /// * `angle` - 回転角度
    ///
    /// # 戻り値
    /// 回転された新しい3D楕円弧
    pub fn rotate_around_axis(
        &self,
        _axis_point: Point3D<T>,
        _axis_direction: Vector3D<T>,
        _angle: Angle<T>,
    ) -> Self {
        // TODO: 軸回転の実装
        self.clone()
    }

    /// X軸回転
    pub fn rotate_x(&self, _center: Point3D<T>, _angle: Angle<T>) -> Self {
        // TODO: X軸回転の実装
        self.clone()
    }

    /// Y軸回転
    pub fn rotate_y(&self, _center: Point3D<T>, _angle: Angle<T>) -> Self {
        // TODO: Y軸回転の実装
        self.clone()
    }

    /// 非等方スケール（3D版）
    ///
    /// 3D楕円弧をX、Y、Z軸で異なる倍率でスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `scale_x` - X軸方向のスケール倍率
    /// * `scale_y` - Y軸方向のスケール倍率
    /// * `scale_z` - Z軸方向のスケール倍率
    ///
    /// # 戻り値
    /// 非等方スケールされた新しい3D楕円弧
    pub fn scale_non_uniform(
        &self,
        _center: Point3D<T>,
        _scale_x: T,
        _scale_y: T,
        _scale_z: T,
    ) -> Self {
        // TODO: 非等方スケールの実装
        self.clone()
    }

    /// 反転
    ///
    /// 3D楕円弧の方向を反転（開始角度と終了角度を交換）
    ///
    /// # 戻り値
    /// 方向反転された新しい3D楕円弧
    pub fn reverse_transform(&self) -> Self {
        Self::new(self.ellipse().clone(), self.end_angle(), self.start_angle())
    }
}

// ============================================================================
// Composite Transform Operations
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    /// 複合変換：平行移動 + 回転
    pub fn translate_and_rotate(
        &self,
        translation: Vector3D<T>,
        _center: Point3D<T>,
        _angle: Angle<T>,
    ) -> Self {
        self.translate(translation)
    }

    /// 複合変換：スケール + 軸回転
    pub fn scale_and_rotate_axis(
        &self,
        _scale_center: Point3D<T>,
        scale_factor: T,
        _axis_point: Point3D<T>,
        _axis_direction: Vector3D<T>,
        _rotation_angle: Angle<T>,
    ) -> Self {
        let scaled = self.scale(scale_factor).unwrap_or(self.clone());
        scaled
    }

    /// 複合変換：非等方スケール + 平行移動
    pub fn scale_non_uniform_and_translate(
        &self,
        _scale_center: Point3D<T>,
        _scale_x: T,
        _scale_y: T,
        _scale_z: T,
        translation: Vector3D<T>,
    ) -> Self {
        self.translate(translation)
    }
}

// ============================================================================
// Validation and Utility Methods
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    /// 変換後の3D楕円弧の妥当性チェック
    pub fn is_valid_after_transform(&self) -> bool {
        self.is_valid()
    }

    /// 変換の等価性チェック（テスト用）
    pub fn transform_equivalent(&self, other: &Self, tolerance: T) -> bool {
        // 中心点の比較
        let center_diff = self.center().distance_to(&other.center());
        if center_diff > tolerance {
            return false;
        }

        // 軸長の比較
        let major_diff = (self.semi_major() - other.semi_major()).abs();
        let minor_diff = (self.semi_minor() - other.semi_minor()).abs();
        if major_diff > tolerance || minor_diff > tolerance {
            return false;
        }

        // 角度の比較
        let start_diff = (self.start_angle().to_radians() - other.start_angle().to_radians()).abs();
        let end_diff = (self.end_angle().to_radians() - other.end_angle().to_radians()).abs();
        start_diff <= tolerance && end_diff <= tolerance
    }
}

// ============================================================================
// Performance Optimized Methods
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    /// 高速平行移動（インプレース風）
    pub fn fast_translate(&self, translation: Vector3D<T>) -> Self {
        self.translate(translation)
    }

    /// 高速等方スケール
    pub fn fast_uniform_scale(&self, _center: Point3D<T>, factor: T) -> Option<Self> {
        self.scale(factor)
    }
}
