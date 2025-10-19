//! Direction3D Extensions 実装
//!
//! Foundation統一システムに基づくDirection3Dの拡張機能
//! Core機能は direction_3d.rs を参照

use crate::Direction3D;
use geo_foundation::Scalar;

// ============================================================================
// Extension Methods (Coreにない新機能のみ)
// ============================================================================

impl<T: Scalar> Direction3D<T> {
    /// XY平面での方位角を取得（XY平面でのX軸からの角度）
    pub fn azimuth_angle(&self) -> geo_foundation::Angle<T> {
        geo_foundation::Angle::from_radians(self.y().atan2(self.x()))
    }

    /// Z軸からの仰角を取得（0 = Z軸方向、π/2 = XY平面）
    pub fn elevation_angle(&self) -> geo_foundation::Angle<T> {
        geo_foundation::Angle::from_radians(self.z().acos())
    }

    /// 他の方向との角度差を計算（3D版）
    pub fn angle_between(&self, other: &Self) -> geo_foundation::Angle<T> {
        let dot = self.dot(other).clamp(-T::ONE, T::ONE);
        geo_foundation::Angle::from_radians(dot.acos())
    }

    /// XY平面での方位角を取得（T型ラジアン - 後方互換性）
    pub fn azimuth_angle_radians(&self) -> T {
        self.azimuth_angle().to_radians()
    }

    /// Z軸からの仰角を取得（T型ラジアン - 後方互換性）
    pub fn elevation_angle_radians(&self) -> T {
        self.elevation_angle().to_radians()
    }

    /// 他の方向との角度差を計算（T型ラジアン - 後方互換性）
    pub fn angle_between_radians(&self, other: &Self) -> T {
        self.angle_between(other).to_radians()
    }

    /// Angle型を使用した方向判定（角度またはラジアン指定可能）
    pub fn is_same_direction_within_angle(&self, other: &Self, angle_tolerance: geo_foundation::Angle<T>) -> bool {
        let angle_diff = self.angle_between(other);
        angle_diff.to_radians() <= angle_tolerance.to_radians()
    }

    /// Angle型を使用した反対方向判定
    pub fn is_opposite_direction_within_angle(&self, other: &Self, angle_tolerance: geo_foundation::Angle<T>) -> bool {
        let angle_diff = self.angle_between(other);
        // πラジアン（180度）に近いかを判定
        (angle_diff.to_radians() - T::PI).abs() <= angle_tolerance.to_radians()
    }
}
