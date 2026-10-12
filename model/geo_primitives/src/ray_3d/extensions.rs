//! Ray3D Extensions 実装
//!
//! Foundation統一システムに基づくRay3Dの拡張機能
//! Core機能は親モジュール（ray_3d.rs）を参照

use crate::{Point3D, Ray3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> std::fmt::Display for Ray3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ray3D(origin: {:?}, direction: {:?})",
            self.origin,
            self.direction_vector()
        )
    }
}

impl<T: Scalar> Ray3D<T> {
    /// 境界ボックスを取得（無限のため起点のみ）
    pub fn bounding_box(&self) -> geo_core::Aabb3D<T> {
        // Ray は無限に延びるため、境界ボックスは起点のみで構成
        let origin = self.origin;
        geo_core::Aabb3D::new(origin, origin)
    }

    /// パラメータ t での接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector3D<T> {
        self.direction_vector()
    }

    /// 点までの距離を計算
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin;
        let projection_length = self.direction_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            // 点が Ray の起点より後ろにある場合
            self.origin.distance_to(point)
        } else {
            // 点を Ray に垂直投影した点までの距離
            let direction_offset = self.direction_vector() * projection_length;
            let projection = Point3D::new(
                self.origin.x() + direction_offset.x(),
                self.origin.y() + direction_offset.y(),
                self.origin.z() + direction_offset.z(),
            );
            point.distance_to(&projection)
        }
    }

    /// Ray 上で指定した点に最も近い点を取得
    pub fn closest_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = *point - self.origin;
        let projection_length = self.direction_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            // 投影が起点より後ろの場合は起点を返す
            self.origin
        } else {
            // 投影した点を返す
            {
                let direction_offset = self.direction_vector() * projection_length;
                Point3D::new(
                    self.origin.x() + direction_offset.x(),
                    self.origin.y() + direction_offset.y(),
                    self.origin.z() + direction_offset.z(),
                )
            }
        }
    }

    /// 指定した点に最も近い Ray 上の点のパラメータを取得
    pub fn closest_parameter(&self, point: &Point3D<T>) -> T {
        let t = self.parameter_for_point(point);
        t.max(T::ZERO) // t < 0 の場合は 0 (起点) を返す
    }

    /// 平行移動
    pub fn translate(&self, offset: &Vector3D<T>) -> Self {
        let new_origin = Point3D::new(
            self.origin.x() + offset.x(),
            self.origin.y() + offset.y(),
            self.origin.z() + offset.z(),
        );
        Self::new(new_origin, self.direction.as_vector()).unwrap()
    }
}
