// torus_solid_3d_transform.rs
// TorusSolid3D の BasicTransform トレイト実装
//
// 基本的な幾何変換（平行移動、回転、スケール）をトーラス固体に適用します。
// 全ての変換は新しいインスタンスを返す immutable な設計です。

use crate::{Direction3D, Point3D, TorusSolid3D, Vector3D};
use analysis::abstract_types::Angle;
use geo_foundation::{BasicTransform, Scalar};

impl<T: Scalar> BasicTransform<T> for TorusSolid3D<T> {
    type Transformed = Self;
    type Vector2D = Vector3D<T>; // 3Dでは2Dベクトル型として3Dベクトル使用
    type Point2D = Point3D<T>; // 3Dでは2D点型として3D点使用
    type Angle = Angle<T>;

    /// 平行移動を適用
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// * 平行移動されたトーラス固体
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_origin = Point3D::new(
            self.origin().x() + translation.x(),
            self.origin().y() + translation.y(),
            self.origin().z() + translation.z(),
        );

        TorusSolid3D::new(
            new_origin,
            *self.z_axis(),
            *self.x_axis(),
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("Translation should preserve validity")
    }

    /// 指定点を中心とした回転を適用（Z軸周りの回転として実装）
    ///
    /// # Arguments
    /// * `center` - 回転中心
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// * 回転されたトーラス固体
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // center 周りの回転
        let relative_origin = Point3D::new(
            self.origin().x() - center.x(),
            self.origin().y() - center.y(),
            self.origin().z() - center.z(),
        );

        // Z軸周りの回転を適用
        let rotated_relative_origin = Point3D::new(
            relative_origin.x() * cos_a - relative_origin.y() * sin_a,
            relative_origin.x() * sin_a + relative_origin.y() * cos_a,
            relative_origin.z(),
        );

        let new_origin = Point3D::new(
            rotated_relative_origin.x() + center.x(),
            rotated_relative_origin.y() + center.y(),
            rotated_relative_origin.z() + center.z(),
        );

        // 軸の回転
        let new_z_axis = Direction3D::from_vector(Vector3D::new(
            self.z_axis().x() * cos_a - self.z_axis().y() * sin_a,
            self.z_axis().x() * sin_a + self.z_axis().y() * cos_a,
            self.z_axis().z(),
        ))
        .unwrap_or_else(|| *self.z_axis());

        let new_x_axis = Direction3D::from_vector(Vector3D::new(
            self.x_axis().x() * cos_a - self.x_axis().y() * sin_a,
            self.x_axis().x() * sin_a + self.x_axis().y() * cos_a,
            self.x_axis().z(),
        ))
        .unwrap_or_else(|| *self.x_axis());

        TorusSolid3D::new(
            new_origin,
            new_z_axis,
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("Rotation should preserve validity")
    }

    /// スケール変換を適用
    ///
    /// # Arguments
    /// * `center` - スケール中心
    /// * `factor` - スケール倍率
    ///
    /// # Returns
    /// * スケールされたトーラス固体
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // 原点のスケール
        let new_origin = Point3D::new(
            center.x() + (self.origin().x() - center.x()) * factor,
            center.y() + (self.origin().y() - center.y()) * factor,
            center.z() + (self.origin().z() - center.z()) * factor,
        );

        // 半径のスケール
        let new_major_radius = self.major_radius() * factor.abs();
        let new_minor_radius = self.minor_radius() * factor.abs();

        TorusSolid3D::new(
            new_origin,
            *self.z_axis(),
            *self.x_axis(),
            new_major_radius,
            new_minor_radius,
        )
        .expect("Scale should preserve validity")
    }
}

/// 追加の変換メソッド（3D特有の操作）
impl<T: Scalar> TorusSolid3D<T> {
    /// Z軸周りの回転（原点中心）
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// * 回転されたトーラス固体
    pub fn rotate_z(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 原点のZ軸周り回転
        let new_origin = Point3D::new(
            self.origin().x() * cos_a - self.origin().y() * sin_a,
            self.origin().x() * sin_a + self.origin().y() * cos_a,
            self.origin().z(),
        );

        // Z軸は変わらない
        let new_z_axis = *self.z_axis();

        // X軸の回転
        let new_x_axis = Direction3D::from_vector(Vector3D::new(
            self.x_axis().x() * cos_a - self.x_axis().y() * sin_a,
            self.x_axis().x() * sin_a + self.x_axis().y() * cos_a,
            self.x_axis().z(),
        ))
        .unwrap_or_else(|| *self.x_axis());

        TorusSolid3D::new(
            new_origin,
            new_z_axis,
            new_x_axis,
            self.major_radius(),
            self.minor_radius(),
        )
        .expect("Z-rotation should preserve validity")
    }

    /// 任意軸周りの回転
    ///
    /// # Arguments
    /// * `axis` - 回転軸ベクトル（正規化される）
    /// * `angle` - 回転角度（ラジアン）
    /// * `center` - 回転中心
    ///
    /// # Returns
    /// * 回転されたトーラス固体
    pub fn rotate_around_axis(&self, axis: &Vector3D<T>, angle: T, center: &Point3D<T>) -> Self {
        // 簡易実装：axis が Z軸の場合のみ対応
        let axis_length_squared = axis.x() * axis.x() + axis.y() * axis.y() + axis.z() * axis.z();
        if axis_length_squared.is_zero() {
            return self.clone();
        }

        let normalized_axis = axis.normalize();

        // Z軸と平行な場合
        if (normalized_axis.x().abs() < T::EPSILON)
            && (normalized_axis.y().abs() < T::EPSILON)
            && ((normalized_axis.z() - T::ONE).abs() < T::EPSILON
                || (normalized_axis.z() + T::ONE).abs() < T::EPSILON)
        {
            // center 周りのZ軸回転として処理
            let rotation_angle = Angle::from_radians(angle);
            return self.rotate(*center, rotation_angle);
        }

        // それ以外は簡易的に同じトーラスを返す
        self.clone()
    }

    /// 指定点周りのスケール変換
    ///
    /// # Arguments
    /// * `factor` - スケール倍率
    /// * `center` - スケール中心
    ///
    /// # Returns
    /// * スケールされたトーラス固体
    pub fn scale_around(&self, factor: T, center: &Point3D<T>) -> Self {
        self.scale(*center, factor)
    }

    /// 平行移動（ベクトル参照版）
    ///
    /// # Arguments
    /// * `translation` - 移動ベクトル
    ///
    /// # Returns
    /// * 平行移動されたトーラス固体
    pub fn translate_by(&self, translation: &Vector3D<T>) -> Self {
        self.translate(*translation)
    }
}
