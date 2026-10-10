//! Ray2D Extensions - 2次元半無限直線の拡張機能
//!
//! Ray2D の高度な幾何演算、変換操作、特殊作成メソッドを提供
//! Core Foundation では提供しない拡張機能のみ

use crate::{Point2D, Ray2D, Vector2D};
use geo_contracts::{Angle, Scalar};

impl<T: Scalar> Ray2D<T> {
    /// Ray を回転
    pub fn rotate(&self, center: &Point2D<T>, angle: Angle<T>) -> Self {
        let rotated_origin = self.origin_internal().rotate_around(center, angle);
        let rotated_direction = self.direction_internal().rotate(angle);
        Self::new(rotated_origin, rotated_direction).unwrap()
    }

    /// 原点を中心に回転
    pub fn rotate_around_origin(&self, angle: Angle<T>) -> Self {
        self.rotate(&Point2D::origin(), angle)
    }

    /// Ray をスケール
    pub fn scale(&self, center: &Point2D<T>, factor: T) -> Self {
        let scaled_origin = *center + (self.origin_internal() - *center) * factor;
        // DerefによりVector2D<T>が得られる
        Self::new(scaled_origin, self.direction_internal().as_vector()).unwrap()
    }

    /// Ray を反転（逆方向の Ray を作成）
    pub fn reverse(&self) -> Self {
        // DerefによりVector2D<T>が得られる
        Self::new(
            self.origin_internal(),
            (-self.direction_internal()).as_vector(),
        )
        .unwrap()
    }

    /// 他の Ray と平行かを判定
    pub fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        let angle = self
            .direction_internal()
            .angle_to(&other.direction_internal());
        angle <= tolerance || (T::PI - angle).abs() <= tolerance
    }

    /// 他の Ray と垂直かを判定
    pub fn is_perpendicular_to(&self, other: &Self, tolerance: T) -> bool {
        let right_angle = T::PI / (T::ONE + T::ONE);
        (self
            .direction_internal()
            .angle_to(&other.direction_internal())
            - right_angle)
            .abs()
            <= tolerance
    }

    /// Ray の角度を取得（X軸正方向からの角度）
    pub fn angle(&self) -> Angle<T> {
        let dir = self.direction_internal();
        Angle::from_radians(dir.y().atan2(dir.x()))
    }

    /// 平行移動（BasicTransformより柔軟）
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        let new_origin = self.origin_internal() + offset;
        Self::new(new_origin, self.direction_internal().as_vector()).unwrap()
    }
}
