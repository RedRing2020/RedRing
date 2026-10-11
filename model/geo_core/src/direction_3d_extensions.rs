//! Direction3D Extensions 実装
//!
//! Foundation統一システムに基づくDirection3Dの拡張機能
//! Core機能は direction_3d.rs を参照

use crate::{Direction3D, Vector3D};
use geo_contracts::{Angle, Scalar};

// Derive traits are handled by #[derive] on the main struct.

impl<T: Scalar> std::fmt::Display for Direction3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Direction3D({}, {}, {})", self.x(), self.y(), self.z())
    }
}

impl<T: Scalar> From<Direction3D<T>> for Vector3D<T> {
    fn from(direction: Direction3D<T>) -> Self {
        direction.as_vector()
    }
}

impl<T: Scalar> Direction3D<T> {
    /// 他の方向との角度差を計算（3D版）
    pub fn angle_between(&self, other: &Self) -> Angle<T> {
        let dot = self.dot(other).clamp(-T::ONE, T::ONE);
        Angle::from_radians(dot.acos())
    }
}
