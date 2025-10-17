//! Direction2D Extensions 実装
//!
//! Foundation統一システムに基づくDirection2Dの拡張機能
//! Core機能は direction_2d.rs を参照

use crate::Direction2D;
use geo_foundation::Scalar;

// ============================================================================
// Extension Methods (Coreにない新機能のみ)
// ============================================================================

impl<T: Scalar> Direction2D<T> {
    /// 角度から方向を作成（ラジアン）
    pub fn from_angle_radians(angle: T) -> Self {
        let x = angle.cos();
        let y = angle.sin();
        Self::new(x, y).unwrap_or_else(|| Self::positive_x())
    }

    /// X軸からの角度を取得（ラジアン）
    pub fn to_angle_radians(&self) -> T {
        self.y().atan2(self.x())
    }

    /// 垂直方向を取得（90度回転） - Coreの rotate_90() とほぼ同じだが別名で提供
    pub fn perpendicular(&self) -> Self {
        self.rotate_90()
    }

    /// 指定角度で回転（ラジアン） - より使いやすい名前で提供
    pub fn rotated_by_angle(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let x = self.x() * cos_a - self.y() * sin_a;
        let y = self.x() * sin_a + self.y() * cos_a;

        Self::new(x, y).unwrap_or_else(|| Self::positive_x())
    }

    /// 2つの方向の間の角度を計算（常に正の値）
    pub fn angle_between(&self, other: &Self) -> T {
        let cross = self.x() * other.y() - self.y() * other.x();
        let dot = self.dot(other);
        cross.atan2(dot).abs()
    }

    /// より良い方向判定（内積を使用、許容誤差付き）
    pub fn is_same_direction_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other);
        dot >= T::ONE - tolerance
    }

    /// 反対方向判定（許容誤差付き）
    pub fn is_opposite_direction_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        let dot = self.dot(other);
        dot <= -T::ONE + tolerance
    }

    /// 単位円上での線形補間
    pub fn slerp(&self, other: &Self, t: T) -> Self {
        let angle_diff = {
            let cross = self.x() * other.y() - self.y() * other.x();
            let dot = self.dot(other);
            cross.atan2(dot)
        };
        let interpolated_angle = self.to_angle_radians() + angle_diff * t;
        Self::from_angle_radians(interpolated_angle)
    }
}
