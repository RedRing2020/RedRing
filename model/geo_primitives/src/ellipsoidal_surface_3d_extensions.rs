//! EllipsoidalSurface3D の拡張機能実装
//!
//! 注意:
//! 衝突判定は `ellipsoidal_surface_3d_collision.rs`、
//! 交差判定は `ellipsoidal_surface_3d_intersection.rs` を参照。

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
