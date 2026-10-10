//! Ellipse3D Extension 機能
//!
//! Extension Foundation パターンに基づく Ellipse3D の拡張実装

use crate::{Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> Ellipse3D<T> {
    /// 楕円上の点での法線ベクトル（3D空間内）
    pub fn normal_at_parameter(&self, t: T) -> Vector3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        // 楕円の接線ベクトル
        let tangent_local = Vector3D::new(
            -self.semi_major_internal() * sin_t,
            self.semi_minor_internal() * cos_t,
            T::ZERO,
        );

        // 局所座標系から世界座標系への変換
        let u_axis = self.major_axis_direction();
        let v_axis = self.minor_axis_direction();

        let tangent_world =
            u_axis.to_vector() * tangent_local.x() + v_axis.to_vector() * tangent_local.y();

        // 楕円平面内の法線（接線に直交）
        let tangent_dir =
            Direction3D::from_vector(tangent_world).unwrap_or(Direction3D::positive_x());
        self.normal().cross(&tangent_dir)
    }

    /// 楕円を平行移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center_internal().x() + offset.x(),
            self.center_internal().y() + offset.y(),
            self.center_internal().z() + offset.z(),
        );

        Self::new(
            new_center,
            self.semi_major_internal(),
            self.semi_minor_internal(),
            self.normal().as_vector(),
            self.major_axis_direction().as_vector(),
        )
        .unwrap() // 既存の楕円から作成するので失敗しない
    }

    /// 楕円を拡大縮小（等方スケール）
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        Some(
            Self::new(
                self.center_internal(),
                self.semi_major_internal() * factor,
                self.semi_minor_internal() * factor,
                self.normal().as_vector(),
                self.major_axis_direction().as_vector(),
            )
            .unwrap(), // 既存の楕円から作成するので失敗しない
        )
    }
}
