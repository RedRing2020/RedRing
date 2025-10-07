//! 改善されたCircle実装
//! ジェネリック、角度構造体、Arc簡素化を適用

use crate::common::constants::{PI, TAU, GEOMETRIC_TOLERANCE};

/// 数値型制約
pub trait Scalar: Copy + Clone + PartialEq + PartialOrd + 
    std::ops::Add<Output = Self> + 
    std::ops::Sub<Output = Self> + 
    std::ops::Mul<Output = Self> + 
    std::ops::Div<Output = Self> +
    std::fmt::Debug + 
    'static
{
    fn pi() -> Self;
    fn tau() -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self;
    fn abs(self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn is_finite(self) -> bool;
    fn from_f64(value: f64) -> Self;
}

impl Scalar for f64 {
    fn pi() -> Self { std::f64::consts::PI }
    fn tau() -> Self { std::f64::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn is_finite(self) -> bool { self.is_finite() }
    fn from_f64(value: f64) -> Self { value }
}

impl Scalar for f32 {
    fn pi() -> Self { std::f32::consts::PI }
    fn tau() -> Self { std::f32::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn is_finite(self) -> bool { self.is_finite() }
    fn from_f64(value: f64) -> Self { value as f32 }
}

/// 角度構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle<T: Scalar> {
    radians: T,
}

impl<T: Scalar> Angle<T> {
    pub fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    pub fn from_degrees(degrees: T) -> Self {
        let deg_to_rad = T::pi() / T::from_f64(180.0);
        Self { radians: degrees * deg_to_rad }
    }

    pub fn radians(&self) -> T {
        self.radians
    }

    pub fn degrees(&self) -> T {
        let rad_to_deg = T::from_f64(180.0) / T::pi();
        self.radians * rad_to_deg
    }

    pub fn normalize(&self) -> Self {
        let tau = T::tau();
        let mut normalized = self.radians;
        while normalized < T::zero() {
            normalized = normalized + tau;
        }
        while normalized >= tau {
            normalized = normalized - tau;
        }
        Self { radians: normalized }
    }

    pub fn difference(&self, other: &Self) -> Self {
        let diff = other.radians - self.radians;
        let tau = T::tau();
        let half_tau = tau / T::two();
        
        if diff > half_tau {
            Self { radians: diff - tau }
        } else if diff < -half_tau {
            Self { radians: diff + tau }
        } else {
            Self { radians: diff }
        }
    }

    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let diff = self.difference(other);
        Self { radians: self.radians + diff.radians * t }
    }
}

/// 統一Circle trait
pub trait Circle<T: Scalar> {
    type Point;
    type Vector;
    type BoundingBox;

    fn center(&self) -> Self::Point;
    fn radius(&self) -> T;

    fn area(&self) -> T {
        T::pi() * self.radius() * self.radius()
    }

    fn circumference(&self) -> T {
        T::tau() * self.radius()
    }

    fn diameter(&self) -> T {
        T::two() * self.radius()
    }

    fn contains_point(&self, point: &Self::Point) -> bool;
    fn on_circumference(&self, point: &Self::Point, tolerance: T) -> bool;
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;
    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector>;
    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector;
    fn bounding_box(&self) -> Self::BoundingBox;
}

/// 3D追加機能
pub trait Circle3D<T: Scalar>: Circle<T> {
    fn normal(&self) -> Self::Vector;
    fn point_on_plane(&self, point: &Self::Point, tolerance: T) -> bool;
    fn project_point_to_plane(&self, point: &Self::Point) -> Self::Point;
    fn to_2d(&self) -> impl Circle<T>;
}

/// Arc trait（Circleベース）
pub trait Arc<T: Scalar> {
    type Circle: Circle<T>;

    fn circle(&self) -> &Self::Circle;
    fn start_angle(&self) -> Angle<T>;
    fn end_angle(&self) -> Angle<T>;

    fn angle_span(&self) -> Angle<T> {
        self.start_angle().difference(&self.end_angle())
    }

    fn arc_length(&self) -> T {
        self.circle().radius() * self.angle_span().radians().abs()
    }

    fn start_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.start_angle())
    }

    fn end_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.end_angle())
    }

    fn point_at_parameter(&self, t: T) -> <Self::Circle as Circle<T>>::Point {
        let interpolated_angle = self.start_angle().lerp(&self.end_angle(), t);
        self.circle().point_at_angle(interpolated_angle)
    }

    fn contains_point(&self, point: &<Self::Circle as Circle<T>>::Point, tolerance: T) -> bool;
}

// 使用例の具象型
pub struct Circle2D<T: Scalar> {
    center: (T, T),
    radius: T,
}

impl<T: Scalar> Circle2D<T> {
    pub fn new(center: (T, T), radius: T) -> Self {
        assert!(radius >= T::zero() && radius.is_finite());
        Self { center, radius }
    }
}

impl<T: Scalar> Circle<T> for Circle2D<T> {
    type Point = (T, T);
    type Vector = (T, T);
    type BoundingBox = ((T, T), (T, T));

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let dx = point.0 - self.center.0;
        let dy = point.1 - self.center.1;
        let distance_squared = dx * dx + dy * dy;
        distance_squared <= self.radius * self.radius
    }

    fn on_circumference(&self, point: &Self::Point, tolerance: T) -> bool {
        let dx = point.0 - self.center.0;
        let dy = point.1 - self.center.1;
        let distance = (dx * dx + dy * dy).sqrt();
        (distance - self.radius).abs() <= tolerance
    }

    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point {
        let rad = angle.radians();
        let x = self.center.0 + self.radius * rad.cos();
        let y = self.center.1 + self.radius * rad.sin();
        (x, y)
    }

    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector> {
        if !self.on_circumference(point, T::from_f64(1e-10)) {
            return None;
        }
        
        let dx = point.0 - self.center.0;
        let dy = point.1 - self.center.1;
        // 接線は法線に垂直
        Some((-dy, dx))
    }

    fn tangent_at_angle(&self, angle: Angle<T>) -> Self::Vector {
        let rad = angle.radians();
        let magnitude = self.radius;
        (-magnitude * rad.sin(), magnitude * rad.cos())
    }

    fn bounding_box(&self) -> Self::BoundingBox {
        let min = (self.center.0 - self.radius, self.center.1 - self.radius);
        let max = (self.center.0 + self.radius, self.center.1 + self.radius);
        (min, max)
    }
}

// Arc実装例（Circleを内包）
pub struct Arc2D<T: Scalar> {
    circle: Circle2D<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

impl<T: Scalar> Arc2D<T> {
    pub fn new(circle: Circle2D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self { circle, start_angle, end_angle }
    }
}

impl<T: Scalar> Arc<T> for Arc2D<T> {
    type Circle = Circle2D<T>;

    fn circle(&self) -> &Self::Circle {
        &self.circle
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    fn contains_point(&self, point: &<Self::Circle as Circle<T>>::Point, tolerance: T) -> bool {
        // 1. 点が円周上にあるかチェック
        if !self.circle.on_circumference(point, tolerance) {
            return false;
        }

        // 2. 点が角度範囲内にあるかチェック
        let dx = point.0 - self.circle.center.0;
        let dy = point.1 - self.circle.center.1;
        let point_angle = Angle::from_radians(dy.atan2(dx));
        
        let start = self.start_angle.normalize();
        let end = self.end_angle.normalize();
        let point_norm = point_angle.normalize();

        // 角度範囲の判定（複雑な場合は省略、実装時に詳細化）
        true // 簡略化
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_circle_f64() {
        let circle = Circle2D::<f64>::new((0.0, 0.0), 5.0);
        assert_eq!(circle.radius(), 5.0);
        assert!((circle.area() - (std::f64::consts::PI * 25.0)).abs() < 1e-10);
    }

    #[test]
    fn test_generic_circle_f32() {
        let circle = Circle2D::<f32>::new((0.0, 0.0), 5.0);
        assert_eq!(circle.radius(), 5.0);
        assert!((circle.area() - (std::f32::consts::PI * 25.0)).abs() < 1e-6);
    }

    #[test]
    fn test_angle_conversion() {
        let angle_deg = Angle::<f64>::from_degrees(90.0);
        let angle_rad = Angle::<f64>::from_radians(std::f64::consts::PI / 2.0);
        assert!((angle_deg.radians() - angle_rad.radians()).abs() < 1e-10);
    }

    #[test]
    fn test_arc_composition() {
        let circle = Circle2D::<f64>::new((0.0, 0.0), 1.0);
        let start = Angle::from_degrees(0.0);
        let end = Angle::from_degrees(90.0);
        let arc = Arc2D::new(circle, start, end);
        
        assert_eq!(arc.circle().radius(), 1.0);
        assert!((arc.arc_length() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
}