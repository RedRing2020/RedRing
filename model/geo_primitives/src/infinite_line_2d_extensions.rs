//! InfiniteLine2D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine2D の拡張実装

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_contracts::default_distance_tolerance;
use geo_contracts::{Angle, Scalar};

impl<T: Scalar> InfiniteLine2D<T> {
    /// X軸に平行な直線を作成（y = y0）
    pub fn horizontal(y: T) -> Self {
        Self::new(Point2D::new(T::ZERO, y), Vector2D::unit_x())
            .expect("Unit vector should always be valid for InfiniteLine2D")
    }

    /// Y軸に平行な直線を作成（x = x0）
    pub fn vertical(x: T) -> Self {
        Self::new(Point2D::new(x, T::ZERO), Vector2D::unit_y())
            .expect("Unit vector should always be valid for InfiniteLine2D")
    }

    /// 傾きを取得（垂直線の場合はNone）
    pub fn slope(&self) -> Option<T> {
        if self.direction_internal().x().abs() <= T::EPSILON {
            None // 垂直線
        } else {
            Some(self.direction_internal().y() / self.direction_internal().x())
        }
    }

    /// Y切片を取得（垂直線の場合はNone）
    pub fn y_intercept(&self) -> Option<T> {
        self.slope()
            .map(|slope| self.point_internal().y() - slope * self.point_internal().x())
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

    /// 直線が同一かを判定
    pub fn is_coincident(&self, other: &Self) -> bool {
        self.is_parallel(other)
            && self.contains_point(&other.point_internal(), default_distance_tolerance::<T>())
    }

    /// 直線が垂直かを判定（角度許容誤差使用）
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_perpendicular_to(&other.direction_internal())
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

    /// 原点周りの回転
    pub fn rotate_around_origin(&self, angle: Angle<T>) -> Self {
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();

        // 点の回転
        let new_x = self.point_internal().x() * cos_a - self.point_internal().y() * sin_a;
        let new_y = self.point_internal().x() * sin_a + self.point_internal().y() * cos_a;
        let new_point = Point2D::new(new_x, new_y);

        // 方向ベクトルの回転
        let dir_x = self.direction_internal().x() * cos_a - self.direction_internal().y() * sin_a;
        let dir_y = self.direction_internal().x() * sin_a + self.direction_internal().y() * cos_a;
        let new_direction = Vector2D::new(dir_x, dir_y);

        Self::new(new_point, new_direction)
            .expect("Rotated direction should always be valid for InfiniteLine2D")
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
