//! ConicalSolid3D の変換操作実装
//!
//! ConicalSolid3D の平行移動、回転、スケール等の変換操作を提供

use super::conical_solid_3d::ConicalSolid3D;
use crate::{Angle, Point3D, Vector3D};
use geo_foundation::{BasicTransform, Scalar};

// ============================================================================
// Core Transform Operations (基本変換)
// ============================================================================

impl<T: Scalar> ConicalSolid3D<T> {
    /// 平行移動
    ///
    /// 円錐を指定したベクトル分だけ移動
    /// 軸、参照方向、サイズは変化せず、中心位置のみ移動
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// 移動後の新しい円錐ソリッド
    pub fn translate(&self, translation: &Vector3D<T>) -> Self {
        Self::new(
            Point3D::new(
                self.center().x() + translation.x(),
                self.center().y() + translation.y(),
                self.center().z() + translation.z(),
            ),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius(),
            self.height(),
        )
        .expect("Translation should preserve valid cone properties")
    }

    /// Z軸周りの回転
    ///
    /// 円錐を指定角度だけZ軸周りに回転
    /// 軸と参照方向が回転行列により変換される
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の新しい円錐ソリッド、無効な値の場合は None
    pub fn rotate_z(&self, angle: T) -> Option<Self> {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        // 回転行列による軸の変換
        let new_axis = Vector3D::new(
            self.axis().x() * cos_theta - self.axis().y() * sin_theta,
            self.axis().x() * sin_theta + self.axis().y() * cos_theta,
            self.axis().z(),
        );

        // 回転行列による参照方向の変換
        let new_ref_direction = Vector3D::new(
            self.ref_direction().x() * cos_theta - self.ref_direction().y() * sin_theta,
            self.ref_direction().x() * sin_theta + self.ref_direction().y() * cos_theta,
            self.ref_direction().z(),
        );

        Self::new(
            self.center(),
            new_axis,
            new_ref_direction,
            self.radius(),
            self.height(),
        )
    }

    /// X軸周りの回転
    pub fn rotate_x_angle(&self, angle: T) -> Option<Self> {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        let new_axis = Vector3D::new(
            self.axis().x(),
            self.axis().y() * cos_theta - self.axis().z() * sin_theta,
            self.axis().y() * sin_theta + self.axis().z() * cos_theta,
        );

        let new_ref_direction = Vector3D::new(
            self.ref_direction().x(),
            self.ref_direction().y() * cos_theta - self.ref_direction().z() * sin_theta,
            self.ref_direction().y() * sin_theta + self.ref_direction().z() * cos_theta,
        );

        Self::new(
            self.center(),
            new_axis,
            new_ref_direction,
            self.radius(),
            self.height(),
        )
    }

    /// Y軸周りの回転
    pub fn rotate_y_angle(&self, angle: T) -> Option<Self> {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();

        let new_axis = Vector3D::new(
            self.axis().x() * cos_theta + self.axis().z() * sin_theta,
            self.axis().y(),
            -self.axis().x() * sin_theta + self.axis().z() * cos_theta,
        );

        let new_ref_direction = Vector3D::new(
            self.ref_direction().x() * cos_theta + self.ref_direction().z() * sin_theta,
            self.ref_direction().y(),
            -self.ref_direction().x() * sin_theta + self.ref_direction().z() * cos_theta,
        );

        Self::new(
            self.center(),
            new_axis,
            new_ref_direction,
            self.radius(),
            self.height(),
        )
    }

    /// 等方スケール
    ///
    /// 円錐の半径と高さを指定倍率でスケール
    /// 軸と参照方向は変化せず、中心も変化しない
    ///
    /// # Arguments
    /// * `factor` - スケール倍率（正の値）
    ///
    /// # Returns
    /// スケール後の新しい円錐ソリッド、無効な倍率の場合は None
    pub fn scale_uniform(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Self::new(
            self.center(),
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            self.radius() * factor,
            self.height() * factor,
        )
    }
}

// ============================================================================
// BasicTransform Trait Implementation (Foundation パターン)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for ConicalSolid3D<T> {
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
        .expect("Rotation should preserve valid cone properties")
    }

    /// 指定中心でのスケール
    fn scale(&self, _center: Self::Point2D, factor: T) -> Self::Transformed {
        self.scale_uniform(factor)
            .expect("Uniform scale should succeed with positive factor")
    }
}
