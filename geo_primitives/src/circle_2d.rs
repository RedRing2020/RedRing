//! 2次元円（Circle2D）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Circle2D の実装

use crate::{Point2D, Vector2D};
use geo_foundation::{
    abstract_types::geometry::foundation::{
        BasicContainment, BasicMetrics, BasicParametric, GeometryFoundation,
    },
    Scalar,
};

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

    /// 中心と半径から円を作成
    pub fn from_center_radius(center: Point2D<T>, radius: T) -> Option<Self> {
        Self::new(center, radius)
    }

    /// 3点から円を作成（外接円）
    pub fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> {
        // 3点が一直線上にある場合は円を作れない
        let v1 = Vector2D::from_points(p1, p2);
        let v2 = Vector2D::from_points(p1, p3);

        let cross = v1.cross(&v2);
        if cross.abs() <= T::EPSILON {
            return None; // 共線点
        }

        // 外接円の中心計算（複素な幾何計算）
        let d1 = p1.x() * p1.x() + p1.y() * p1.y();
        let d2 = p2.x() * p2.x() + p2.y() * p2.y();
        let d3 = p3.x() * p3.x() + p3.y() * p3.y();

        let two = T::ONE + T::ONE;
        let aux1 = d1 * (p2.y() - p3.y()) + d2 * (p3.y() - p1.y()) + d3 * (p1.y() - p2.y());
        let aux2 = d1 * (p3.x() - p2.x()) + d2 * (p1.x() - p3.x()) + d3 * (p2.x() - p1.x());
        let div = two
            * (p1.x() * (p2.y() - p3.y())
                + p2.x() * (p3.y() - p1.y())
                + p3.x() * (p1.y() - p2.y()));

        if div.abs() <= T::EPSILON {
            return None;
        }

        let center = Point2D::new(aux1 / div, aux2 / div);
        let radius = center.distance_to(&p1);

        Some(Self { center, radius })
    }

    /// 単位円を作成（原点中心、半径1）
    pub fn unit_circle() -> Self {
        Self {
            center: Point2D::new(T::ZERO, T::ZERO),
            radius: T::ONE,
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

    /// 直径を取得
    pub fn diameter(&self) -> T {
        let two = T::ONE + T::ONE;
        self.radius * two
    }

    /// 円周の長さを取得
    pub fn circumference(&self) -> T {
        let tau = T::TAU;
        tau * self.radius
    }

    /// 面積を取得
    pub fn area(&self) -> T {
        let pi = T::from_f64(std::f64::consts::PI);
        pi * self.radius * self.radius
    }

    /// 指定角度での点を取得（ラジアン）
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point2D::new(
            self.center.x() + self.radius * cos_a,
            self.center.y() + self.radius * sin_a,
        )
    }

    /// 指定パラメータでの点を取得（0-1の範囲で一周）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let tau = T::TAU;
        let angle = t * tau;
        self.point_at_angle(angle)
    }

    /// 指定パラメータでの接線ベクトルを取得
    pub fn tangent_at_parameter(&self, t: T) -> Vector2D<T> {
        let tau = T::TAU;
        let angle = t * tau;
        let sin_a = angle.sin();
        let cos_a = angle.cos();
        // 接線ベクトルは半径方向に垂直
        Vector2D::new(-sin_a, cos_a).normalize()
    }

    /// 点が円上にあるかを判定
    pub fn contains_point_on_circle(&self, point: &Point2D<T>, tolerance: T) -> bool {
        let distance = self.center.distance_to(point);
        (distance - self.radius).abs() <= tolerance
    }

    /// 点が円内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point2D<T>) -> bool {
        let distance = self.center.distance_to(point);
        distance <= self.radius
    }

    /// 点から円への最短距離を計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let distance_to_center = self.center.distance_to(point);
        if distance_to_center <= self.radius {
            T::ZERO // 点が円内部にある
        } else {
            distance_to_center - self.radius
        }
    }

    /// 円を指定倍率でスケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor > T::ZERO {
            Some(Self {
                center: self.center,
                radius: self.radius * factor,
            })
        } else {
            None
        }
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self {
            center: self.center + offset,
            radius: self.radius,
        }
    }

    /// 円を指定点に移動
    pub fn move_to(&self, new_center: Point2D<T>) -> Self {
        Self {
            center: new_center,
            radius: self.radius,
        }
    }

    /// 他の円と交差するかを判定
    pub fn intersects_circle(&self, other: &Self) -> bool {
        let distance = self.center.distance_to(&other.center);
        let sum_radii = self.radius + other.radius;
        let diff_radii = (self.radius - other.radius).abs();

        distance <= sum_radii && distance >= diff_radii
    }

    /// 他の円を完全に含むかを判定
    pub fn contains_circle(&self, other: &Self) -> bool {
        let distance = self.center.distance_to(&other.center);
        distance + other.radius <= self.radius
    }

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> crate::BBox2D<T> {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        crate::BBox2D::new(min_point, max_point)
    }

    /// 3次元円に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::Circle3D<T> {
        crate::Circle3D::new(
            self.center.to_3d(),
            Vector2D::new(T::ZERO, T::ZERO).to_3d_with_z(T::ONE), // Z軸法線
            self.radius,
        )
        .unwrap()
    }

    /// 3次元円に拡張（指定Z値平面）
    pub fn to_3d_at_z(&self, z: T) -> crate::Circle3D<T> {
        crate::Circle3D::new(
            self.center.to_3d_with_z(z),
            Vector2D::new(T::ZERO, T::ZERO).to_3d_with_z(T::ONE), // Z軸法線
            self.radius,
        )
        .unwrap()
    }
}

// ============================================================================
// Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> GeometryFoundation<T> for Circle2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        Circle2D::bounding_box(self)
    }
}

impl<T: Scalar> BasicMetrics<T> for Circle2D<T> {
    fn length(&self) -> Option<T> {
        Some(self.circumference())
    }

    fn area(&self) -> Option<T> {
        Some(Circle2D::area(self))
    }

    fn perimeter(&self) -> Option<T> {
        Some(self.circumference())
    }
}

impl<T: Scalar> BasicContainment<T> for Circle2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point_inside(point)
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point_on_circle(point, tolerance)
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        Circle2D::distance_to_point(self, point)
    }
}

impl<T: Scalar> BasicParametric<T> for Circle2D<T> {
    fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE) // 0-1の範囲で一周
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        Circle2D::point_at_parameter(self, t)
    }

    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        Circle2D::tangent_at_parameter(self, t)
    }
}
