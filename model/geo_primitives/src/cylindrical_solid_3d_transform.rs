//! CylindricalSolid3D Transform Operations
//!
//! STEP準拠円柱ソリッドの変換操作実装
//!
//! **作成日: 2025年10月31日**
//! **最終更新: 2025年10月31日**
//!
//! ## 実装内容
//! - 平行移動：center の移動
//! - 回転：軸と参照方向の回転
//! - スケール：半径・高さのスケーリング
//! - BasicTransform トレイト実装（Foundation パターン準拠）
//!
//! ## STEP準拠円柱ソリッド変換の特性
//! - 軸と参照方向の直交性保持：変換後も axis ⊥ ref_direction を維持
//! - 正規化保持：軸と参照方向が単位ベクトルのまま
//! - 右手系保持：Y軸 = Z軸 × X軸 関係を維持
//! - 幾何学的整合性：半径・高さの正の値保持
//! - ソリッド特性保持：体積比例、内部判定整合性

use crate::{CylindricalSolid3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// Basic Transform Operations
// ============================================================================

impl<T: Scalar> CylindricalSolid3D<T> {
    /// 平行移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 平行移動後の円柱ソリッド
    ///
    /// # Note
    /// 軸方向、参照方向、半径、高さは変更されず、center のみ移動
    pub fn translate(&self, translation: &Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center().x() + translation.x(),
            self.center().y() + translation.y(),
            self.center().z() + translation.z(),
        );

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.height(),
        )
        .expect("Translation should preserve valid cylinder properties")
    }

    /// 均等スケール変換
    ///
    /// # Arguments
    /// * `scale_factor` - スケール係数（正の値）
    ///
    /// # Returns
    /// スケール変換後の円柱ソリッド、無効な係数の場合は None
    ///
    /// # Note
    /// 中心、軸方向、参照方向はそのまま、半径と高さを scale_factor 倍
    pub fn scale_uniform(&self, scale_factor: T) -> Option<Self> {
        if scale_factor <= T::ZERO {
            return None;
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius() * scale_factor,
            self.height() * scale_factor,
        )
    }

    /// 非均等スケール変換
    ///
    /// # Arguments
    /// * `radius_scale` - 半径のスケール係数（正の値）
    /// * `height_scale` - 高さのスケール係数（正の値）
    ///
    /// # Returns
    /// 非均等スケール変換後の円柱ソリッド、無効な係数の場合は None
    ///
    /// # Note
    /// 中心、軸方向、参照方向はそのまま、半径と高さを個別にスケール
    pub fn scale_non_uniform(&self, radius_scale: T, height_scale: T) -> Option<Self> {
        if radius_scale <= T::ZERO || height_scale <= T::ZERO {
            return None;
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius() * radius_scale,
            self.height() * height_scale,
        )
    }

    /// Z軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// Z軸周りに回転した円柱ソリッド
    ///
    /// # Note
    /// 原点を中心にZ軸周りに回転。center, axis, ref_direction が回転される
    pub fn rotate_z(&self, angle: &Angle<T>) -> Self {
        self.rotate_around_axis(
            &Point3D::new(T::ZERO, T::ZERO, T::ZERO),
            &Vector3D::new(T::ZERO, T::ZERO, T::ONE),
            angle,
        )
        .expect("Z-axis rotation should always be valid")
    }

    /// 任意軸周りの回転
    ///
    /// # Arguments
    /// * `rotation_center` - 回転中心点
    /// * `rotation_axis` - 回転軸（正規化される）
    /// * `angle` - 回転角度
    ///
    /// # Returns
    /// 回転後の円柱ソリッド、回転軸が無効な場合は None
    ///
    /// # Note
    /// center, axis, ref_direction を回転軸周りに回転
    /// 半径、高さは不変
    pub fn rotate_around_axis(
        &self,
        rotation_center: &Point3D<T>,
        rotation_axis: &Vector3D<T>,
        angle: &Angle<T>,
    ) -> Option<Self> {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        // 回転軸の正規化
        let axis_norm = rotation_axis.magnitude();
        if axis_norm.is_zero() {
            return None;
        }
        let normalized_axis = *rotation_axis / axis_norm;

        // Rodrigues の回転公式のヘルパー関数
        let rotate_vector = |v: &Vector3D<T>| -> Vector3D<T> {
            let k = normalized_axis;
            let v_parallel = k * v.dot(&k);
            let v_perpendicular = *v - v_parallel;
            let w = k.cross(v);

            v_parallel + v_perpendicular * cos_theta + w * sin_theta
        };

        // 中心点の回転
        let center_vec = Vector3D::new(
            self.center().x() - rotation_center.x(),
            self.center().y() - rotation_center.y(),
            self.center().z() - rotation_center.z(),
        );
        let rotated_center_vec = rotate_vector(&center_vec);
        let new_center = Point3D::new(
            rotation_center.x() + rotated_center_vec.x(),
            rotation_center.y() + rotated_center_vec.y(),
            rotation_center.z() + rotated_center_vec.z(),
        );

        // 軸の回転
        let rotated_axis = rotate_vector(&self.axis().as_vector());

        // 参照方向の回転
        let rotated_ref_direction = rotate_vector(&self.ref_direction().as_vector());

        Self::new(
            new_center,
            rotated_axis,
            rotated_ref_direction,
            self.radius(),
            self.height(),
        )
    }
}

// ============================================================================
// BasicTransform Trait Implementation (Foundation パターン)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for CylindricalSolid3D<T> {
    type Transformed = Self;
    type Vector2D = Vector3D<T>; // 3Dでは2Dベクトル型として3Dベクトル使用
    type Point2D = Point3D<T>; // 3Dでは2D点型として3D点使用
    type Angle = Angle<T>;

    /// 平行移動（Foundation パターン）
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        self.translate(&translation)
    }

    /// 指定中心での回転（Z軸周り回転として実装）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        // 簡単な原点周りZ軸回転として実装
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        // 回転行列による変換（Z軸周り）
        let rotate_point = |p: &Point3D<T>| -> Point3D<T> {
            Point3D::new(
                p.x() * cos_theta - p.y() * sin_theta,
                p.x() * sin_theta + p.y() * cos_theta,
                p.z(),
            )
        };

        let rotate_vector = |v: &Vector3D<T>| -> Vector3D<T> {
            Vector3D::new(
                v.x() * cos_theta - v.y() * sin_theta,
                v.x() * sin_theta + v.y() * cos_theta,
                v.z(),
            )
        };

        // center 周りの回転
        let relative_center = Point3D::new(
            self.center().x() - center.x(),
            self.center().y() - center.y(),
            self.center().z() - center.z(),
        );
        let rotated_relative = rotate_point(&relative_center);
        let new_center = Point3D::new(
            center.x() + rotated_relative.x(),
            center.y() + rotated_relative.y(),
            center.z() + rotated_relative.z(),
        );

        // 軸と参照方向を回転
        let rotated_axis = rotate_vector(&self.axis().as_vector());
        let rotated_ref_direction = rotate_vector(&self.ref_direction().as_vector());

        Self::new(
            new_center,
            rotated_axis,
            rotated_ref_direction,
            self.radius(),
            self.height(),
        )
        .expect("Rotation should preserve valid cylinder properties")
    }

    /// 指定中心でのスケール
    fn scale(&self, _center: Self::Point2D, factor: T) -> Self::Transformed {
        self.scale_uniform(factor)
            .expect("Uniform scale should succeed with positive factor")
    }
}
