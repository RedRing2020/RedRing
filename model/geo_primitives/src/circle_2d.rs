//! 2次元円（Circle2D）のCore実装
//!
//! Foundation統一システムに基づくCircle2Dの必須機能のみ
//! STEP (ISO 10303) 準拠の ref_direction フィールドでArc変換に対応

use crate::{Direction2D, Point2D};
use geo_foundation::Scalar;

/// 2次元円
///
/// STEP準拠でref_direction（参照方向）を持ち、Arc2Dへの変換とトリム操作が可能
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
    /// 参照方向（STEP準拠）- 角度0度の方向を定義
    ref_direction: Direction2D<T>,
}

impl<T: Scalar> Circle2D<T> {
    /// 新しい円を作成（デフォルトでX軸正方向を参照方向とする）
    pub fn new(center: Point2D<T>, radius: T) -> Option<Self> {
        if radius > T::ZERO {
            Some(Self {
                center,
                radius,
                ref_direction: Direction2D::positive_x(),
            })
        } else {
            None
        }
    }

    /// 参照方向を指定して円を作成
    pub fn new_with_ref_direction(
        center: Point2D<T>,
        radius: T,
        ref_direction: Direction2D<T>,
    ) -> Option<Self> {
        if radius > T::ZERO {
            Some(Self {
                center,
                radius,
                ref_direction,
            })
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

    /// 参照方向を取得
    pub fn ref_direction(&self) -> Direction2D<T> {
        self.ref_direction
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

    /// 点円（半径がゼロに近い）かどうか
    pub fn is_point(&self) -> bool {
        self.radius <= T::EPSILON
    }

    /// 単位円かどうか
    pub fn is_unit_circle(&self) -> bool {
        (self.radius - T::ONE).abs() <= T::EPSILON
    }

    /// 原点中心かどうか
    pub fn is_centered_at_origin(&self) -> bool {
        self.center.x().abs() <= T::EPSILON && self.center.y().abs() <= T::EPSILON
    }

    /// 点が円周上にあるか判定
    pub fn point_on_circumference(&self, point: Point2D<T>) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance = (dx * dx + dy * dy).sqrt();
        (distance - self.radius).abs() <= T::EPSILON
    }

    /// 点に最も近い円周上の点を取得
    pub fn closest_point_to(&self, point: Point2D<T>) -> Point2D<T> {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= T::EPSILON {
            // 点が中心にある場合、任意の円周上の点を返す
            Point2D::new(self.center.x() + self.radius, self.center.y())
        } else {
            let scale = self.radius / distance;
            Point2D::new(self.center.x() + dx * scale, self.center.y() + dy * scale)
        }
    }

    /// 点における円のパラメータを取得
    pub fn parameter_at_point(&self, point: Point2D<T>) -> Option<T> {
        if !self.point_on_circumference(point) {
            return None;
        }

        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let angle = dy.atan2(dx);

        // 0-1の範囲に正規化
        let parameter = if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        } / T::TAU;

        Some(parameter)
    }

    /// 2つの円の距離
    pub fn distance_to_circle(&self, other: &Self) -> T {
        let center_distance = {
            let dx = other.center.x() - self.center.x();
            let dy = other.center.y() - self.center.y();
            (dx * dx + dy * dy).sqrt()
        };

        let radii_sum = self.radius + other.radius;

        if center_distance >= radii_sum {
            // 円が外部にある
            center_distance - radii_sum
        } else if center_distance <= (self.radius - other.radius).abs() {
            // 一方が他方の内部にある
            (self.radius - other.radius).abs() - center_distance
        } else {
            // 円が交差している
            T::ZERO
        }
    }
}
