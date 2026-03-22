//! EllipsoidalSurface3D の拡張機能実装
//!
//! 混在 collision/intersection の正本は `geo_algorithms` 側で管理する。

use crate::EllipsoidalSurface3D;
use geo_contracts::Scalar;

impl<T: Scalar> EllipsoidalSurface3D<T> {
    /// 半径3成分を返す。
    pub fn semi_axes(&self) -> (T, T, T) {
        (
            self.a_radius_internal(),
            self.b_radius_internal(),
            self.c_radius_internal(),
        )
    }

    /// 回転楕円体が扁平型かどうかを返す。
    pub fn is_oblate_spheroid(&self) -> bool {
        let (a, b, c) = self.semi_axes();
        (a - b).abs() < T::EPSILON && c < a
    }

    /// 回転楕円体が長球型かどうかを返す。
    pub fn is_prolate_spheroid(&self) -> bool {
        let (a, b, c) = self.semi_axes();
        (a - b).abs() < T::EPSILON && c > a
    }
}
