//! ConicalSurface3D の基本変換操作（BasicTransform）実装

use crate::{ConicalSurface3D, Point3D, Vector3D};
use analysis::abstract_types::Angle;
use geo_foundation::{BasicTransform, Scalar};

impl<T: Scalar> BasicTransform<T> for ConicalSurface3D<T> {
    type Transformed = Self;
    type Vector2D = Vector3D<T>; // 3Dでは2Dベクトル型として3Dベクトル使用
    type Point2D = Point3D<T>; // 3Dでは2D点型として3D点使用
    type Angle = Angle<T>;

    /// 平行移動（Foundation パターン）
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
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
            self.semi_angle(),
        )
        .expect("平行移動は常に有効な円錐サーフェスを生成する")
    }

    /// 指定中心での回転（Z軸周り回転として実装）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // center 周りの回転
        let relative_center = Point3D::new(
            self.center().x() - center.x(),
            self.center().y() - center.y(),
            self.center().z() - center.z(),
        );

        let rotated_relative = Point3D::new(
            cos_a * relative_center.x() - sin_a * relative_center.y(),
            sin_a * relative_center.x() + cos_a * relative_center.y(),
            relative_center.z(),
        );

        let new_center = Point3D::new(
            center.x() + rotated_relative.x(),
            center.y() + rotated_relative.y(),
            center.z() + rotated_relative.z(),
        );

        // 軸方向の回転
        let axis_vec = self.axis().as_vector();
        let new_axis_vec = Vector3D::new(
            cos_a * axis_vec.x() - sin_a * axis_vec.y(),
            sin_a * axis_vec.x() + cos_a * axis_vec.y(),
            axis_vec.z(),
        );

        // 参照方向の回転
        let ref_vec = self.ref_direction().as_vector();
        let new_ref_vec = Vector3D::new(
            cos_a * ref_vec.x() - sin_a * ref_vec.y(),
            sin_a * ref_vec.x() + cos_a * ref_vec.y(),
            ref_vec.z(),
        );

        Self::new(
            new_center,
            new_axis_vec,
            new_ref_vec,
            self.radius(),
            self.semi_angle(),
        )
        .expect("Z軸回転は常に有効な円錐サーフェスを生成する")
    }

    /// 指定中心でのスケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // center からの相対位置
        let relative_center = Vector3D::new(
            self.center().x() - center.x(),
            self.center().y() - center.y(),
            self.center().z() - center.z(),
        );

        // スケール後の新しい中心
        let scaled_relative = Vector3D::new(
            relative_center.x() * factor,
            relative_center.y() * factor,
            relative_center.z() * factor,
        );

        let new_center = Point3D::new(
            center.x() + scaled_relative.x(),
            center.y() + scaled_relative.y(),
            center.z() + scaled_relative.z(),
        );

        // 半径をスケール（半頂角は不変）
        let new_radius = self.radius() * factor.abs();

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            self.semi_angle(),
        )
        .expect("スケールは常に有効な円錐サーフェスを生成する")
    }
}

// ============================================================================
// 追加の変換メソッド（古いAPIとの互換性維持）
// ============================================================================

impl<T: Scalar> ConicalSurface3D<T> {
    /// Z軸周りの回転
    ///
    /// # Arguments
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の円錐サーフェス
    pub fn rotate_z(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心点の回転
        let new_center = Point3D::new(
            cos_a * self.center().x() - sin_a * self.center().y(),
            sin_a * self.center().x() + cos_a * self.center().y(),
            self.center().z(),
        );

        // 軸方向の回転
        let axis_vec = self.axis().as_vector();
        let new_axis_vec = Vector3D::new(
            cos_a * axis_vec.x() - sin_a * axis_vec.y(),
            sin_a * axis_vec.x() + cos_a * axis_vec.y(),
            axis_vec.z(),
        );

        // 参照方向の回転
        let ref_vec = self.ref_direction().as_vector();
        let new_ref_vec = Vector3D::new(
            cos_a * ref_vec.x() - sin_a * ref_vec.y(),
            sin_a * ref_vec.x() + cos_a * ref_vec.y(),
            ref_vec.z(),
        );

        Self::new(
            new_center,
            new_axis_vec,
            new_ref_vec,
            self.radius(),
            self.semi_angle(),
        )
        .expect("Z軸回転は常に有効な円錐サーフェスを生成する")
    }

    /// 任意軸周りの回転
    ///
    /// # Arguments
    /// * `axis` - 回転軸
    /// * `angle` - 回転角度（ラジアン）
    ///
    /// # Returns
    /// 回転後の円錐サーフェス
    pub fn rotate_axis(&self, axis: Vector3D<T>, angle: T) -> Self {
        // ロドリゲスの回転公式を使用
        let k = axis.normalize();
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let one_minus_cos = T::ONE - cos_a;

        // 回転行列の成分
        let r11 = cos_a + k.x() * k.x() * one_minus_cos;
        let r12 = k.x() * k.y() * one_minus_cos - k.z() * sin_a;
        let r13 = k.x() * k.z() * one_minus_cos + k.y() * sin_a;

        let r21 = k.y() * k.x() * one_minus_cos + k.z() * sin_a;
        let r22 = cos_a + k.y() * k.y() * one_minus_cos;
        let r23 = k.y() * k.z() * one_minus_cos - k.x() * sin_a;

        let r31 = k.z() * k.x() * one_minus_cos - k.y() * sin_a;
        let r32 = k.z() * k.y() * one_minus_cos + k.x() * sin_a;
        let r33 = cos_a + k.z() * k.z() * one_minus_cos;

        // 中心点の回転
        let center_vec = Vector3D::new(self.center().x(), self.center().y(), self.center().z());
        let new_center = Point3D::new(
            r11 * center_vec.x() + r12 * center_vec.y() + r13 * center_vec.z(),
            r21 * center_vec.x() + r22 * center_vec.y() + r23 * center_vec.z(),
            r31 * center_vec.x() + r32 * center_vec.y() + r33 * center_vec.z(),
        );

        // 軸方向の回転
        let axis_vec = self.axis().as_vector();
        let new_axis_vec = Vector3D::new(
            r11 * axis_vec.x() + r12 * axis_vec.y() + r13 * axis_vec.z(),
            r21 * axis_vec.x() + r22 * axis_vec.y() + r23 * axis_vec.z(),
            r31 * axis_vec.x() + r32 * axis_vec.y() + r33 * axis_vec.z(),
        );

        // 参照方向の回転
        let ref_vec = self.ref_direction().as_vector();
        let new_ref_vec = Vector3D::new(
            r11 * ref_vec.x() + r12 * ref_vec.y() + r13 * ref_vec.z(),
            r21 * ref_vec.x() + r22 * ref_vec.y() + r23 * ref_vec.z(),
            r31 * ref_vec.x() + r32 * ref_vec.y() + r33 * ref_vec.z(),
        );

        Self::new(
            new_center,
            new_axis_vec,
            new_ref_vec,
            self.radius(),
            self.semi_angle(),
        )
        .expect("軸回転は常に有効な円錐サーフェスを生成する")
    }

    /// 一様スケール
    ///
    /// # Arguments
    /// * `factor` - スケール倍率
    ///
    /// # Returns
    /// スケール後の円錐サーフェス
    pub fn scale_uniform(&self, factor: T) -> Self {
        // スケール後の中心点
        let new_center = Point3D::new(
            self.center().x() * factor,
            self.center().y() * factor,
            self.center().z() * factor,
        );

        // 半径をスケール（半頂角は不変）
        let new_radius = self.radius() * factor.abs();

        Self::new(
            new_center,
            self.axis().as_vector(),
            self.ref_direction().as_vector(),
            new_radius,
            self.semi_angle(),
        )
        .expect("一様スケールは常に有効な円錐サーフェスを生成する")
    }

    /// 非一様スケール
    ///
    /// # Arguments
    /// * `scale_x` - X方向のスケール倍率
    /// * `scale_y` - Y方向のスケール倍率
    /// * `scale_z` - Z方向のスケール倍率
    ///
    /// # Returns
    /// スケール後の円錐サーフェス（近似）
    ///
    /// # Note
    /// 非一様スケールは円錐を楕円錐に変形するため、
    /// ここでは平均倍率を使用した近似を行う
    pub fn scale_non_uniform(&self, scale_x: T, scale_y: T, scale_z: T) -> Self {
        // 中心点の非一様スケール
        let new_center = Point3D::new(
            self.center().x() * scale_x,
            self.center().y() * scale_y,
            self.center().z() * scale_z,
        );

        // 軸方向の非一様スケール
        let axis_vec = self.axis().as_vector();
        let scaled_axis = Vector3D::new(
            axis_vec.x() * scale_x,
            axis_vec.y() * scale_y,
            axis_vec.z() * scale_z,
        );

        // 参照方向の非一様スケール
        let ref_vec = self.ref_direction().as_vector();
        let scaled_ref = Vector3D::new(
            ref_vec.x() * scale_x,
            ref_vec.y() * scale_y,
            ref_vec.z() * scale_z,
        );

        // 半径の近似スケール（XY平面の平均倍率）
        let xy_scale_avg = (scale_x.abs() + scale_y.abs()) / (T::ONE + T::ONE);
        let new_radius = self.radius() * xy_scale_avg;

        Self::new(
            new_center,
            scaled_axis,
            scaled_ref,
            new_radius,
            self.semi_angle(),
        )
        .expect("非一様スケールでも有効な円錐サーフェスが生成される")
    }

    /// 平面による反射
    ///
    /// # Arguments
    /// * `plane_point` - 平面上の点
    /// * `plane_normal` - 平面の法線ベクトル
    ///
    /// # Returns
    /// 反射後の円錐サーフェス
    pub fn reflect(&self, plane_point: Point3D<T>, plane_normal: Vector3D<T>) -> Self {
        let n = plane_normal.normalize();
        let two = T::ONE + T::ONE;

        // 中心点の反射
        let center_to_plane = Vector3D::new(
            self.center().x() - plane_point.x(),
            self.center().y() - plane_point.y(),
            self.center().z() - plane_point.z(),
        );
        let distance =
            center_to_plane.x() * n.x() + center_to_plane.y() * n.y() + center_to_plane.z() * n.z();
        let new_center = Point3D::new(
            self.center().x() - two * distance * n.x(),
            self.center().y() - two * distance * n.y(),
            self.center().z() - two * distance * n.z(),
        );

        // 軸方向の反射
        let axis_vec = self.axis().as_vector();
        let axis_dot = axis_vec.x() * n.x() + axis_vec.y() * n.y() + axis_vec.z() * n.z();
        let new_axis_vec = Vector3D::new(
            axis_vec.x() - two * axis_dot * n.x(),
            axis_vec.y() - two * axis_dot * n.y(),
            axis_vec.z() - two * axis_dot * n.z(),
        );

        // 参照方向の反射
        let ref_vec = self.ref_direction().as_vector();
        let ref_dot = ref_vec.x() * n.x() + ref_vec.y() * n.y() + ref_vec.z() * n.z();
        let new_ref_vec = Vector3D::new(
            ref_vec.x() - two * ref_dot * n.x(),
            ref_vec.y() - two * ref_dot * n.y(),
            ref_vec.z() - two * ref_dot * n.z(),
        );

        Self::new(
            new_center,
            new_axis_vec,
            new_ref_vec,
            self.radius(),
            self.semi_angle(),
        )
        .expect("反射は常に有効な円錐サーフェスを生成する")
    }
}
