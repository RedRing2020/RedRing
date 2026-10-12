//! Direction2D Extensions 実装
//!
//! Foundation統一システムに基づくDirection2Dの拡張機能
//! Core機能は direction_2d.rs を参照

use crate::Direction2D;
use analysis::abstract_types::Scalar;

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

    /// 2つの方向の間の角度を計算（常に正の値）
    pub fn angle_between(&self, other: Self) -> T {
        let cross = self.x() * other.y() - self.y() * other.x();
        let dot = self.dot(other);
        cross.atan2(dot).abs()
    }

    /// 単位円上での線形補間
    pub fn slerp(&self, other: Self, t: T) -> Self {
        let angle_diff = {
            let cross = self.x() * other.y() - self.y() * other.x();
            let dot = self.dot(other);
            cross.atan2(dot)
        };
        let interpolated_angle = self.to_angle_radians() + angle_diff * t;
        Self::from_angle_radians(interpolated_angle)
    }
}
