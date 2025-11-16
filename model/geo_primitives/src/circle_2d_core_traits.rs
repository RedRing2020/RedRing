//! Circle2D Core Traits Implementation
//!
//! Circle2D構造体へのCore Traitsの実装
//! Point/Vector Core Traitsと同じパターンで統一的に実装

use crate::{Circle2D, Direction2D, Point2D};
use geo_foundation::Scalar;

// ============================================================================
// Circle2D Core Traits - 直接実装アプローチ
// ============================================================================

/// Circle2D生成のためのConstructorトレイト
pub trait Circle2DConstructor<T: Scalar> {
    fn new(center: Point2D<T>, radius: T) -> Option<Self>
    where
        Self: Sized;
    fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
    fn from_center_point(center: Point2D<T>, point_on_circle: Point2D<T>) -> Option<Self>
    where
        Self: Sized;
    fn unit_circle() -> Self
    where
        Self: Sized;
}

/// Circle2D基本プロパティ取得トレイト
pub trait Circle2DProperties<T: Scalar> {
    fn center(&self) -> Point2D<T>;
    fn radius(&self) -> T;
    fn ref_direction(&self) -> Direction2D<T>;
    fn diameter(&self) -> T;
    fn is_point(&self) -> bool;
    fn is_unit_circle(&self) -> bool;
    fn is_centered_at_origin(&self) -> bool;
    fn dimension(&self) -> u32;
}

/// Circle2D計量機能トレイト
pub trait Circle2DMeasure<T: Scalar> {
    fn circumference(&self) -> T;
    fn area(&self) -> T;
    fn contains_point(&self, point: Point2D<T>) -> bool;
    fn point_on_circumference(&self, point: Point2D<T>) -> bool;
    fn distance_to_point(&self, point: Point2D<T>) -> T;
    fn point_at_parameter(&self, t: T) -> Point2D<T>;
    fn parameter_at_point(&self, point: Point2D<T>) -> Option<T>;
    fn distance_to_circle(&self, other: &Self) -> T;
    fn closest_point_to(&self, point: Point2D<T>) -> Point2D<T>;
}

// ============================================================================
// 1. Constructor Trait Implementation
// ============================================================================

impl<T: Scalar> Circle2DConstructor<T> for Circle2D<T> {
    fn new(center: Point2D<T>, radius: T) -> Option<Self> {
        Circle2D::new(center, radius)
    }

    fn from_three_points(p1: Point2D<T>, p2: Point2D<T>, p3: Point2D<T>) -> Option<Self> {
        // 三点から円の中心と半径を計算
        let ax = p1.x();
        let ay = p1.y();
        let bx = p2.x();
        let by = p2.y();
        let cx = p3.x();
        let cy = p3.y();

        // 行列式で三点が一直線上でないかチェック
        let d = T::from_f64(2.0) * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by));
        if d.abs() <= T::EPSILON {
            // 三点が一直線上
            return None;
        }

        // 外心を計算
        let ux = ax * ax + ay * ay;
        let uy = bx * bx + by * by;
        let uz = cx * cx + cy * cy;

        let center_x = (ux * (by - cy) + uy * (cy - ay) + uz * (ay - by)) / d;
        let center_y = (ux * (cx - bx) + uy * (ax - cx) + uz * (bx - ax)) / d;

        let center = Point2D::new(center_x, center_y);

        // 半径を計算（第一点までの距離）
        let dx = center_x - ax;
        let dy = center_y - ay;
        let radius = (dx * dx + dy * dy).sqrt();

        // デフォルトのX軸正方向で円を作成
        Circle2D::new(center, radius)
    }

    fn from_center_point(center: Point2D<T>, point_on_circle: Point2D<T>) -> Option<Self> {
        let dx = point_on_circle.x() - center.x();
        let dy = point_on_circle.y() - center.y();
        let radius = (dx * dx + dy * dy).sqrt();

        Circle2D::new(center, radius)
    }

    fn unit_circle() -> Self {
        Circle2D::new(Point2D::new(T::ZERO, T::ZERO), T::ONE).expect("単位円の作成は必ず成功する")
    }
}

// ============================================================================
// 2. Properties Trait Implementation
// ============================================================================

// ============================================================================
// 2. Properties Trait Implementation
// ============================================================================

impl<T: Scalar> Circle2DProperties<T> for Circle2D<T> {
    fn center(&self) -> Point2D<T> {
        self.center()
    }

    fn radius(&self) -> T {
        self.radius()
    }

    fn ref_direction(&self) -> Direction2D<T> {
        self.ref_direction()
    }

    fn diameter(&self) -> T {
        self.radius() + self.radius()
    }

    fn is_point(&self) -> bool {
        self.radius() <= T::EPSILON
    }

    fn is_unit_circle(&self) -> bool {
        (self.radius() - T::ONE).abs() <= T::EPSILON
    }

    fn is_centered_at_origin(&self) -> bool {
        let center = self.center();
        center.x().abs() <= T::EPSILON && center.y().abs() <= T::EPSILON
    }

    fn dimension(&self) -> u32 {
        1
    }
}

// ============================================================================
// 3. Measure Trait Implementation
// ============================================================================

// ============================================================================
// 3. Measure Trait Implementation
// ============================================================================

impl<T: Scalar> Circle2DMeasure<T> for Circle2D<T> {
    fn circumference(&self) -> T {
        T::TAU * self.radius()
    }

    fn area(&self) -> T {
        T::PI * self.radius() * self.radius()
    }

    fn contains_point(&self, point: Point2D<T>) -> bool {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        let distance_squared = dx * dx + dy * dy;
        distance_squared < self.radius() * self.radius()
    }

    fn point_on_circumference(&self, point: Point2D<T>) -> bool {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        let distance_squared = dx * dx + dy * dy;
        let radius_squared = self.radius() * self.radius();
        (distance_squared - radius_squared).abs() <= T::EPSILON * radius_squared
    }

    fn distance_to_point(&self, point: Point2D<T>) -> T {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        let center_distance = (dx * dx + dy * dy).sqrt();
        center_distance - self.radius()
    }

    fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = T::TAU * t;
        Point2D::new(
            self.center().x() + self.radius() * angle.cos(),
            self.center().y() + self.radius() * angle.sin(),
        )
    }

    fn parameter_at_point(&self, point: Point2D<T>) -> Option<T> {
        if !self.point_on_circumference(point) {
            return None;
        }

        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();

        let angle = dy.atan2(dx);
        let normalized_angle = if angle < T::ZERO {
            angle + T::TAU
        } else {
            angle
        };

        Some(normalized_angle / T::TAU)
    }

    fn distance_to_circle(&self, other: &Self) -> T {
        let dx = other.center().x() - self.center().x();
        let dy = other.center().y() - self.center().y();
        let center_distance = (dx * dx + dy * dy).sqrt();

        center_distance - self.radius() - other.radius()
    }

    fn closest_point_to(&self, point: Point2D<T>) -> Point2D<T> {
        let dx = point.x() - self.center().x();
        let dy = point.y() - self.center().y();
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= T::EPSILON {
            // 点が中心にある場合は任意の円周上の点を返す
            Point2D::new(self.center().x() + self.radius(), self.center().y())
        } else {
            let scale = self.radius() / distance;
            Point2D::new(
                self.center().x() + dx * scale,
                self.center().y() + dy * scale,
            )
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::tolerance_migration::DefaultTolerances;

    fn assert_near<T: Scalar>(a: T, b: T) {
        assert!(
            (a - b).abs() < DefaultTolerances::distance::<T>(),
            "Values not close enough: {} != {}",
            a.to_f64(),
            b.to_f64()
        );
    }

    #[test]
    fn test_constructor_basic() {
        let center = Point2D::new(1.0, 2.0);
        let circle = Circle2D::new(center, 3.0).unwrap();

        assert_eq!(circle.center(), center);
        assert_near(circle.radius(), 3.0);
    }

    #[test]
    fn test_constructor_invalid_radius() {
        let center = Point2D::new(0.0, 0.0);
        assert!(Circle2D::new(center, 0.0).is_none());
        assert!(Circle2D::new(center, -1.0).is_none());
    }

    #[test]
    fn test_from_three_points() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(2.0, 0.0);
        let p3 = Point2D::new(1.0, 1.0);

        let circle = Circle2D::from_three_points(p1, p2, p3).unwrap();

        // 三点が円周上にあることを確認
        assert!(circle.point_on_circumference(p1));
        assert!(circle.point_on_circumference(p2));
        assert!(circle.point_on_circumference(p3));
    }

    #[test]
    fn test_from_three_points_collinear() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(1.0, 0.0);
        let p3 = Point2D::new(2.0, 0.0);

        // 一直線上の三点では円は作れない
        assert!(Circle2D::from_three_points(p1, p2, p3).is_none());
    }

    #[test]
    fn test_unit_circle() {
        let circle = Circle2D::<f64>::unit_circle();

        assert!(circle.is_unit_circle());
        assert!(circle.is_centered_at_origin());
        assert_near(circle.radius(), 1.0);
    }

    #[test]
    fn test_circumference_and_area() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

        assert_near(circle.circumference(), 4.0 * std::f64::consts::PI);
        assert_near(circle.area(), 4.0 * std::f64::consts::PI);
    }

    #[test]
    fn test_point_containment() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 2.0).unwrap();

        assert!(circle.contains_point(Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(Point2D::new(1.0, 1.0))); // 内部
        assert!(!circle.contains_point(Point2D::new(3.0, 0.0))); // 外部

        assert!(circle.point_on_circumference(Point2D::new(2.0, 0.0))); // 円周上
        assert!(circle.point_on_circumference(Point2D::new(0.0, 2.0))); // 円周上
    }

    #[test]
    fn test_parameter_functions() {
        let circle = Circle2D::new(Point2D::new(0.0, 0.0), 1.0).unwrap();

        // パラメータ0で(1,0)
        let p0 = circle.point_at_parameter(0.0);
        assert_near(p0.x(), 1.0);
        assert_near(p0.y(), 0.0);

        // パラメータ0.25で(0,1)
        let p25 = circle.point_at_parameter(0.25);
        assert_near(p25.x(), 0.0);
        assert_near(p25.y(), 1.0);

        // 逆変換テスト
        let param = circle.parameter_at_point(p0).unwrap();
        assert_near(param, 0.0);
    }
}
