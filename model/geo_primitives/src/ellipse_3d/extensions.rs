//! Ellipse3D Extension 機能
//!
//! Extension Foundation パターンに基づく Ellipse3D の拡張実装

use crate::{Ellipse3D, Point3D, Vector3D};
use geo_contracts::Scalar;

impl<T: Scalar> Ellipse3D<T> {
    /// 楕円を平行移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        let new_center = Point3D::new(
            self.center.x() + offset.x(),
            self.center.y() + offset.y(),
            self.center.z() + offset.z(),
        );

        Self::new(
            new_center,
            self.semi_major_axis,
            self.semi_minor_axis,
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
                self.center,
                self.semi_major_axis * factor,
                self.semi_minor_axis * factor,
                self.normal().as_vector(),
                self.major_axis_direction().as_vector(),
            )
            .unwrap(), // 既存の楕円から作成するので失敗しない
        )
    }
}
