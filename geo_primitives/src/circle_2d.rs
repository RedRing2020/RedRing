//! 2次元円（Circle2D）のCore実装
//!
//! Foundation統一システムに基づくCircle2Dの必須機能のみ

use crate::Point2D;
use geo_foundation::Scalar;

/// 2次元円
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
}

impl<T: Scalar> Circle2D<T> {
    /// 新しい円を作成
    pub fn new(center: Point2D<T>, radius: T) -> Option<Self> {
        if radius > T::ZERO {
            Some(Self { center, radius })
        } else {
            None
        }
    }

    /// 中心を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// バウンディングボックスを取得
    pub fn bounding_box(&self) -> (Point2D<T>, Point2D<T>) {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        (min_point, max_point)
    }

    /// 円周の長さ
    pub fn circumference(&self) -> T {
        T::TAU * self.radius
    }

    /// 円の面積
    pub fn area(&self) -> T {
        T::PI * self.radius * self.radius
    }

    /// 点が円内部にあるか判定
    pub fn contains_point(&self, point: Point2D<T>) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance_squared = dx * dx + dy * dy;
        distance_squared < self.radius * self.radius
    }

    /// パラメータでの点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = T::TAU * t;
        Point2D::new(
            self.center.x() + self.radius * angle.cos(),
            self.center.y() + self.radius * angle.sin(),
        )
    }

    /// 点から円周への距離
    pub fn distance_to_point(&self, point: Point2D<T>) -> T {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let center_distance = (dx * dx + dy * dy).sqrt();
        (center_distance - self.radius).abs()
    }
}
