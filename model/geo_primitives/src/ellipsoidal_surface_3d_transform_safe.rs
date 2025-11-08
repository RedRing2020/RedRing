// ellipsoidal_surface_3d_transform_safe.rs
// EllipsoidalSurface3D の SafeTransform 実装（エラーハンドリング付き）

use crate::{EllipsoidalSurface3D, Point3D, Vector3D};
use geo_foundation::{Angle, SafeTransform, Scalar, TransformError};

impl<T: Scalar> SafeTransform<T> for EllipsoidalSurface3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;

    /// 安全な平行移動
    fn safe_translate(&self, offset: Self::Vector) -> Result<Self, TransformError> {
        // オフセットベクトルの有効性をチェック
        if !offset.x().is_finite() || !offset.y().is_finite() || !offset.z().is_finite() {
            return Err(TransformError::InvalidGeometry(
                "平行移動オフセットに無限値または非数値が含まれています".to_string(),
            ));
        }

        let new_center = *self.center() + offset;

        Ok(EllipsoidalSurface3D::new(
            new_center,
            self.semi_axis_a(),
            self.semi_axis_b(),
            self.semi_axis_c(),
            *self.z_axis(),
            *self.x_axis(),
        )
        .expect("有効な平行移動"))
    }

    /// 安全なスケール変換
    fn safe_scale(
        &self,
        center: Self::Point,
        factors: Self::Vector,
    ) -> Result<Self, TransformError> {
        // スケール倍率の有効性をチェック
        if !factors.x().is_finite() || !factors.y().is_finite() || !factors.z().is_finite() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率に無限値または非数値が含まれています".to_string(),
            ));
        }

        if factors.x().is_zero() || factors.y().is_zero() || factors.z().is_zero() {
            return Err(TransformError::InvalidScaleFactor(
                "スケール倍率がゼロです".to_string(),
            ));
        }

        if factors.x() < T::ZERO || factors.y() < T::ZERO || factors.z() < T::ZERO {
            return Err(TransformError::InvalidScaleFactor(
                "負のスケール倍率は許可されていません".to_string(),
            ));
        }

        // 新しい半軸長を計算
        let new_a = self.semi_axis_a() * factors.x();
        let new_b = self.semi_axis_b() * factors.y();
        let new_c = self.semi_axis_c() * factors.z();

        // 結果が有効な楕円体制約 (a >= b >= c > 0) を満たすかチェック
        if !(new_a >= new_b && new_b >= new_c && new_c > T::ZERO) {
            return Err(TransformError::InvalidGeometry(
                "スケール後の軸長が楕円体制約を満たしません".to_string(),
            ));
        }

        // 中心のスケール変換
        let scaled_center = Point3D::new(
            center.x() + (self.center().x() - center.x()) * factors.x(),
            center.y() + (self.center().y() - center.y()) * factors.y(),
            center.z() + (self.center().z() - center.z()) * factors.z(),
        );

        Ok(EllipsoidalSurface3D::new(
            scaled_center,
            new_a,
            new_b,
            new_c,
            *self.z_axis(),
            *self.x_axis(),
        )
        .expect("有効なスケール変換"))
    }

    /// 安全な回転変換
    fn safe_rotate(
        &self,
        center: Self::Point,
        axis: Self::Vector,
        angle: Angle<T>,
    ) -> Result<Self, TransformError> {
        // 回転軸の有効性をチェック
        if !axis.x().is_finite() || !axis.y().is_finite() || !axis.z().is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転軸に無限値または非数値が含まれています".to_string(),
            ));
        }

        // 回転軸がゼロベクトルかチェック
        if axis.norm().is_zero() {
            return Err(TransformError::ZeroVector(
                "回転軸がゼロベクトルです".to_string(),
            ));
        }

        // 角度の有効性をチェック
        if !angle.to_radians().is_finite() {
            return Err(TransformError::InvalidRotation(
                "回転角度に無限値または非数値が含まれています".to_string(),
            ));
        }

        // 簡単な Z軸周りの回転として実装（中心座標系を考慮）
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心の回転（回転中心からの相対位置で計算）
        let current_center = *self.center();
        let relative_center = Point3D::new(
            current_center.x() - center.x(),
            current_center.y() - center.y(),
            current_center.z() - center.z(),
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

        // X軸の回転
        let current_x_axis = self.x_axis();
        let rotated_x_vec = Vector3D::new(
            cos_a * current_x_axis.x() - sin_a * current_x_axis.y(),
            sin_a * current_x_axis.x() + cos_a * current_x_axis.y(),
            current_x_axis.z(),
        );
        let new_x_axis = crate::Direction3D::from_vector(rotated_x_vec)
            .ok_or_else(|| TransformError::InvalidRotation("軸の回転に失敗しました".to_string()))?;

        Ok(EllipsoidalSurface3D::new(
            new_center,
            self.semi_axis_a(),
            self.semi_axis_b(),
            self.semi_axis_c(),
            *self.z_axis(),
            new_x_axis,
        )
        .expect("有効な回転変換"))
    }
}
