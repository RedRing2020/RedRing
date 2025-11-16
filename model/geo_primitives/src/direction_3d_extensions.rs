//! Direction3D Extensions 実装
//!
//! Foundation統一システムに基づくDirection3Dの拡張機能
//! Core機能は direction_3d.rs を参照

use crate::{Direction3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Core trait implementations (moved from core)
// ============================================================================

// Derive traits now handled by #[derive] macro in main struct definition

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

// ============================================================================
// Extended vector operations (additional methods)
// ============================================================================

impl<T: Scalar> Direction3D<T> {
    /// 他の方向と直交しているかをチェック
    pub fn is_orthogonal_to(&self, other: &Self, tolerance: T) -> bool {
        self.dot(other).abs() <= tolerance
    }

    /// 他の方向と同じ方向かをチェック（高精度）
    pub fn is_same_direction(&self, other: &Self, tolerance: T) -> bool {
        let dot_product = self.dot(other);
        dot_product >= T::ONE - tolerance
    }

    /// 他の方向と反対方向かをチェック（高精度）
    pub fn is_opposite_direction(&self, other: &Self, tolerance: T) -> bool {
        let dot_product = self.dot(other);
        dot_product <= -T::ONE + tolerance
    }

    /// XY平面での方位角を取得（XY平面でのX軸からの角度）
    pub fn azimuth_angle(&self) -> Angle<T> {
        Angle::from_radians(self.y().atan2(self.x()))
    }

    /// Z軸からの仰角を取得（0 = Z軸方向、π/2 = XY平面）
    pub fn elevation_angle(&self) -> Angle<T> {
        Angle::from_radians(self.z().acos())
    }

    /// 他の方向との角度差を計算（3D版）
    pub fn angle_between(&self, other: &Self) -> Angle<T> {
        let dot = self.dot(other).clamp(-T::ONE, T::ONE);
        Angle::from_radians(dot.acos())
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
    pub fn is_same_direction_within_angle(&self, other: &Self, angle_tolerance: Angle<T>) -> bool {
        let angle_diff = self.angle_between(other);
        angle_diff.to_radians() <= angle_tolerance.to_radians()
    }

    /// Angle型を使用した反対方向判定
    pub fn is_opposite_direction_within_angle(
        &self,
        other: &Self,
        angle_tolerance: Angle<T>,
    ) -> bool {
        let angle_diff = self.angle_between(other);
        // πラジアン（180度）に近いかを判定
        (angle_diff.to_radians() - T::PI).abs() <= angle_tolerance.to_radians()
    }
}
