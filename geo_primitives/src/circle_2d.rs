//! 2次元円（Circle2D）の Core 実装
//!
//! Core Foundation パターンに基づく Circle2D の必須機能のみ
//! 拡張機能は circle_2d_extensions.rs を参照

use crate::{Point2D, Vector2D};
use geo_foundation::{
    abstract_types::geometry::core_foundation::{
        BasicContainment, BasicMetrics, BasicParametric, CoreFoundation,
    },
    Scalar,
};

/// 2次元円
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Circle2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

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

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 中心を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    // ========================================================================
    // Core Metrics Methods
    // ========================================================================

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

    // ========================================================================
    // Core Containment Methods
    // ========================================================================

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

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================

    /// 指定パラメータでの点を取得（0-1の範囲で一周）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let tau = T::TAU;
        let angle = t * tau;
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point2D::new(
            self.center.x() + self.radius * cos_a,
            self.center.y() + self.radius * sin_a,
        )
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

    // ========================================================================
    // Core Bounding Box Method
    // ========================================================================

    /// 境界ボックスを取得
    pub fn bounding_box(&self) -> crate::BBox2D<T> {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        crate::BBox2D::new(min_point, max_point)
    }
}

// ============================================================================
// Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for Circle2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
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
