//! InfiniteLine2D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine2D の拡張実装

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_contracts::Scalar;

impl<T: Scalar> InfiniteLine2D<T> {
    /// X軸に平行な直線を作成（y = y0）
    pub fn horizontal(y: T) -> Self {
        Self::new(Point2D::new(T::ZERO, y), Vector2D::unit_x())
            .expect("Unit vector should always be valid for InfiniteLine2D")
    }

    /// X軸との角度を取得（ラジアン）
    pub fn angle(&self) -> T {
        self.direction_internal().angle().to_radians()
    }

    /// 直線が平行かを判定（角度許容誤差使用）
    pub fn is_parallel(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
    }

    /// 他の直線と同じ直線かを判定
    pub fn is_same_line(&self, other: &Self, tolerance: T) -> bool {
        self.is_parallel(other) && self.contains_point(&other.point_internal(), tolerance)
    }

    /// 直線を平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self::new(self.point_internal() + offset, *self.direction_internal())
            .expect("Existing direction should always be valid for InfiniteLine2D")
    }

    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self::new(self.point_internal(), *(-self.direction_internal()))
            .expect("Negated direction should always be valid for InfiniteLine2D")
    }

    /// 3次元無限直線に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::InfiniteLine3D<T> {
        crate::InfiniteLine3D::new(
            self.point_internal().to_3d(),
            self.direction_internal().to_3d(),
        )
        .unwrap()
    }
}
