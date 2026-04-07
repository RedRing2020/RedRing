//! 2次元円（Circle2D）のCore実装
//!
//! Foundation統一システムに基づくCircle2Dの必須機能のみ
//! STEP (ISO 10303) 準拠の ref_direction フィールドでArc変換に対応

use crate::{Direction2D, Point2D};
use geo_contracts::{default_distance_tolerance, default_kernel_numerical_zero_tolerance};
use geo_contracts::{
    Circle2DConstructor, Circle2DContainment, Circle2DDerived, Circle2DDistance,
    Circle2DEvaluation, Circle2DProjection, Circle2DProperties, CrossDistance, PrimitiveKind,
    PrimitiveMetadata, Scalar,
};

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

impl<T: Scalar> PrimitiveMetadata for Circle2D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Circle
    }
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

    /// 中心を取得（内部用）
    pub(crate) fn center_internal(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得（内部用）
    pub(crate) fn radius_internal(&self) -> T {
        self.radius
    }

    /// 参照方向を取得（内部用）
    pub(crate) fn ref_direction_internal(&self) -> Direction2D<T> {
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
        let distance_squared = point.distance_squared_to(&self.center);
        distance_squared < self.radius * self.radius
    }

    /// Primitive 局所座標系の local angle parameter `t` (`0 <= t <= 2π`) で円周上の点を取得
    ///
    /// `point_at_angle` と同じ local angle domain を使う core evaluation 入口として扱う。
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        Point2D::new(
            self.center.x() + self.radius * t.cos(),
            self.center.y() + self.radius * t.sin(),
        )
    }

    /// 点から円周への距離
    pub fn distance_to_point(&self, point: Point2D<T>) -> T {
        let center_distance = point.distance_to(&self.center);
        (center_distance - self.radius).abs()
    }

    /// 点円（半径がゼロに近い）かどうか
    pub fn is_point(&self) -> bool {
        self.radius <= default_distance_tolerance::<T>()
    }

    /// 点が円周上にあるか判定
    pub fn point_on_circumference(&self, point: Point2D<T>) -> bool {
        let distance = point.distance_to(&self.center);
        (distance - self.radius).abs() <= default_distance_tolerance::<T>()
    }

    /// 点に最も近い円周上の点を取得
    pub fn closest_point_to(&self, point: Point2D<T>) -> Point2D<T> {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let distance = point.distance_to(&self.center);

        if distance <= default_kernel_numerical_zero_tolerance::<T>() {
            // 点が中心にある場合、任意の円周上の点を返す
            Point2D::new(self.center.x() + self.radius, self.center.y())
        } else {
            let scale = self.radius / distance;
            Point2D::new(self.center.x() + dx * scale, self.center.y() + dy * scale)
        }
    }

    /// 円周上の点を Circle core の local angle parameter (`0 <= t < 2π`) へ写像する
    pub fn parameter_at_point(&self, point: Point2D<T>) -> Option<T> {
        if !self.point_on_circumference(point) {
            return None;
        }

        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let angle = dy.atan2(dx);

        // 0-2π の local angle parameter に正規化
        let parameter = if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        };

        Some(parameter)
    }

    /// 2つの円の距離
    pub fn distance_to_circle(&self, other: &Self) -> T {
        let center_distance = self.center.distance_to(&other.center);

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

    /// 3点から円を作成する内部メソッド
    fn from_three_points_internal(
        point1: Point2D<T>,
        point2: Point2D<T>,
        point3: Point2D<T>,
    ) -> Option<Self> {
        // 3点が一直線上にある場合は円が定義できない
        let dx1 = point2.x() - point1.x();
        let dy1 = point2.y() - point1.y();
        let dx2 = point3.x() - point2.x();
        let dy2 = point3.y() - point2.y();

        let cross = dx1 * dy2 - dy1 * dx2;
        if cross.abs() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None; // 3点が一直線上
        }

        // 外心を計算
        let d = T::from_f64(2.0)
            * (point1.x() * (point2.y() - point3.y())
                + point2.x() * (point3.y() - point1.y())
                + point3.x() * (point1.y() - point2.y()));

        if d.abs() <= default_kernel_numerical_zero_tolerance::<T>() {
            return None;
        }

        let p1_sq = point1.x() * point1.x() + point1.y() * point1.y();
        let p2_sq = point2.x() * point2.x() + point2.y() * point2.y();
        let p3_sq = point3.x() * point3.x() + point3.y() * point3.y();

        let cx = (p1_sq * (point2.y() - point3.y())
            + p2_sq * (point3.y() - point1.y())
            + p3_sq * (point1.y() - point2.y()))
            / d;
        let cy = (p1_sq * (point3.x() - point2.x())
            + p2_sq * (point1.x() - point3.x())
            + p3_sq * (point2.x() - point1.x()))
            / d;

        let center = Point2D::new(cx, cy);

        // 半径を計算
        let radius = center.distance_to(&point1);

        Self::new(center, radius)
    }
}

impl<T: Scalar> Circle2DConstructor<T> for Circle2D<T> {
    fn new(center: (T, T), radius: T) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        Self::new(center_point, radius)
    }

    fn new_with_ref_direction(center: (T, T), radius: T, ref_direction: (T, T)) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        let ref_dir = Direction2D::new(ref_direction.0, ref_direction.1)?;
        Self::new_with_ref_direction(center_point, radius, ref_dir)
    }

    fn unit_circle() -> Self {
        let center = Point2D::origin();
        Self::new(center, T::ONE).unwrap()
    }

    fn from_center_and_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self> {
        let center_point = Point2D::new(center.0, center.1);
        let point = Point2D::new(point_on_circle.0, point_on_circle.1);

        let radius = point.distance_to(&center_point);

        if radius <= T::ZERO {
            None
        } else {
            Self::new(center_point, radius)
        }
    }

    fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self> {
        let point1 = Point2D::new(p1.0, p1.1);
        let point2 = Point2D::new(p2.0, p2.1);
        let point3 = Point2D::new(p3.0, p3.1);

        Self::from_three_points_internal(point1, point2, point3)
    }

    fn centered_at_origin(radius: T) -> Option<Self> {
        Self::new(Point2D::origin(), radius)
    }
}

impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    fn center(&self) -> (T, T) {
        let c = self.center_internal();
        (c.x(), c.y())
    }

    fn radius(&self) -> T {
        self.radius_internal()
    }

    fn ref_direction(&self) -> (T, T) {
        let r = self.ref_direction_internal();
        (r.x(), r.y())
    }

    fn diameter(&self) -> T {
        let r = self.radius_internal();
        r + r
    }

    fn dimension(&self) -> u32 {
        2
    }

    fn is_unit_circle(&self) -> bool {
        (self.radius_internal() - T::ONE).abs() <= default_distance_tolerance::<T>()
    }

    fn is_centered_at_origin(&self) -> bool {
        let c = self.center_internal();
        c.x().abs() <= default_distance_tolerance::<T>()
            && c.y().abs() <= default_distance_tolerance::<T>()
    }

    fn is_degenerate(&self) -> bool {
        self.radius_internal() <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Circle2DDerived<T> for Circle2D<T> {
    fn circumference(&self) -> T {
        Circle2D::circumference(self)
    }

    fn area(&self) -> T {
        Circle2D::area(self)
    }
}

impl<T: Scalar> Circle2DContainment<T> for Circle2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        Circle2D::contains_point(self, p)
    }

    fn point_on_circumference(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        Circle2D::point_on_circumference(self, p)
    }
}

impl<T: Scalar> Circle2DDistance<T> for Circle2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        Circle2D::distance_to_point(self, p)
    }
}

impl<T: Scalar> Circle2DProjection<T> for Circle2D<T> {
    fn closest_point_to(&self, point: (T, T)) -> (T, T) {
        let p = Point2D::new(point.0, point.1);
        let closest = Circle2D::closest_point_to(self, p);
        (closest.x(), closest.y())
    }
}

impl<T: Scalar> Circle2DEvaluation<T> for Circle2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let point = Circle2D::point_at_parameter(self, t);
        (point.x(), point.y())
    }
}

impl<T: Scalar> CrossDistance<T, Self> for Circle2D<T> {
    fn distance_to(&self, other: &Self) -> T {
        Circle2D::distance_to_circle(self, other)
    }
}
