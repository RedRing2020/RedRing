//! 2D Circle implementation
//!
//! 2次元平面における円の具体的な実装

use crate::geometry2d::{BBox2D, Point2D, Vector2D};
use crate::traits::Circle2D;
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;

/// 2D平面上の円を表現する構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    center: Point2D,
    radius: f64,
}

impl Circle {
    /// 新しい円を作成
    ///
    /// # Arguments
    /// * `center` - 円の中心点
    /// * `radius` - 円の半径（正の値）
    ///
    /// # Panics
    /// 半径が負の値またはNaNの場合にパニックする
    pub fn new(center: Point2D, radius: f64) -> Self {
        assert!(
            radius >= 0.0 && radius.is_finite(),
            "半径は非負の有限値である必要があります"
        );
        Self { center, radius }
    }

    /// 原点を中心とする円を作成
    ///
    /// # Arguments
    /// * `radius` - 円の半径（正の値）
    pub fn origin_circle(radius: f64) -> Self {
        Self::new(Point2D::new(0.0, 0.0), radius)
    }

    /// 単位円（半径1、原点中心）を作成
    pub fn unit_circle() -> Self {
        Self::new(Point2D::new(0.0, 0.0), 1.0)
    }

    /// 3点を通る円を作成
    ///
    /// # Arguments
    /// * `p1`, `p2`, `p3` - 円周上の3点
    ///
    /// # Returns
    /// 3点を通る円、または3点が一直線上にある場合は`None`
    pub fn from_three_points(p1: Point2D, p2: Point2D, p3: Point2D) -> Option<Self> {
        // 3点が一直線上にないかチェック
        let v1 = Vector2D::new(p2.x() - p1.x(), p2.y() - p1.y());
        let v2 = Vector2D::new(p3.x() - p1.x(), p3.y() - p1.y());

        let cross = v1.cross_2d(&v2);
        if cross.abs() < GEOMETRIC_TOLERANCE {
            return None; // 3点が一直線上にある
        }

        // 外心を計算
        let d = 2.0 * cross;
        let ux = (v2.y() * v1.length_squared() - v1.y() * v2.length_squared()) / d;
        let uy = (v1.x() * v2.length_squared() - v2.x() * v1.length_squared()) / d;

        let center = Point2D::new(p1.x() + ux, p1.y() + uy);
        let radius = center.distance_to(&p1);

        Some(Self::new(center, radius))
    }

    /// 円が退化しているか（半径が0）を判定
    pub fn is_degenerate(&self) -> bool {
        self.radius < GEOMETRIC_TOLERANCE
    }

    /// 指定された点までの最短距離を取得
    /// 円周までの距離（内部の点では負の値）
    pub fn distance_to_point(&self, point: &Point2D) -> f64 {
        let center_distance = self.center.distance_to(point);
        center_distance - self.radius
    }

    /// 円を指定倍率で拡大縮小
    pub fn scale(&self, factor: f64) -> Self {
        assert!(
            factor >= 0.0 && factor.is_finite(),
            "拡大縮小係数は非負の有限値である必要があります"
        );
        Self::new(self.center, self.radius * factor)
    }

    /// 円を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector2D) -> Self {
        let new_center = Point2D::new(self.center.x() + vector.x(), self.center.y() + vector.y());
        Self::new(new_center, self.radius)
    }

    /// 境界上の点を含む点の包含判定（許容誤差付き）
    pub fn contains_point_on_boundary(&self, point: &Point2D, tolerance: f64) -> bool {
        let distance = self.distance_to_point(point).abs();
        distance <= tolerance
    }

    /// 他の円との交差判定
    pub fn intersects_with_circle(&self, other: &Circle) -> bool {
        let distance = self.center.distance_to(&other.center);
        let sum_radii = self.radius + other.radius;
        let diff_radii = (self.radius - other.radius).abs();
        
        distance <= sum_radii && distance >= diff_radii
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }
}

impl Circle2D for Circle {
    type Point = Point2D;
    type Vector = Vector2D;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> f64 {
        self.radius
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let distance_squared =
            (point.x() - self.center.x()).powi(2) + (point.y() - self.center.y()).powi(2);
        distance_squared <= self.radius.powi(2) + GEOMETRIC_TOLERANCE
    }

    fn on_circumference(&self, point: &Self::Point, tolerance: f64) -> bool {
        let distance = self.center.distance_to(point);
        (distance - self.radius).abs() <= tolerance
    }

    fn point_at_angle(&self, angle: f64) -> Self::Point {
        let x = self.center.x() + self.radius * angle.cos();
        let y = self.center.y() + self.radius * angle.sin();
        Point2D::new(x, y)
    }

    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector> {
        if !self.on_circumference(point, GEOMETRIC_TOLERANCE) {
            return None;
        }

        // 中心から点への方向ベクトル
        let radial = Vector2D::new(point.x() - self.center.x(), point.y() - self.center.y());

        // 接線ベクトルは法線ベクトルに垂直
        Some(radial.perpendicular())
    }

    fn tangent_at_angle(&self, angle: f64) -> Self::Vector {
        // 接線ベクトルは半径ベクトルに垂直
        Vector2D::new(-self.radius * angle.sin(), self.radius * angle.cos())
    }

    fn bounding_box(&self) -> (Self::Point, Self::Point) {
        let min_point = Point2D::new(self.center.x() - self.radius, self.center.y() - self.radius);
        let max_point = Point2D::new(self.center.x() + self.radius, self.center.y() + self.radius);
        (min_point, max_point)
    }
}

impl From<Circle> for BBox2D {
    fn from(circle: Circle) -> Self {
        let (min, max) = circle.bounding_box();
        BBox2D::new((min.x(), min.y()), (max.x(), max.y()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{PI, TAU};

    #[test]
    fn test_circle_creation() {
        let center = Point2D::new(1.0, 2.0);
        let circle = Circle::new(center, 3.0);

        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), 3.0);
        assert_eq!(circle.area(), PI * 9.0);
        assert_eq!(circle.circumference(), TAU * 3.0);
    }

    #[test]
    fn test_unit_circle() {
        let circle = Circle::unit_circle();

        assert_eq!(circle.center(), Point2D::new(0.0, 0.0));
        assert_eq!(circle.radius(), 1.0);
        assert_eq!(circle.area(), PI);
        assert_eq!(circle.circumference(), TAU);
    }

    #[test]
    fn test_contains_point() {
        let circle = Circle::new(Point2D::new(0.0, 0.0), 5.0);

        assert!(circle.contains_point(&Point2D::new(0.0, 0.0))); // 中心
        assert!(circle.contains_point(&Point2D::new(3.0, 4.0))); // 内部
        assert!(circle.contains_point(&Point2D::new(5.0, 0.0))); // 円周上
        assert!(!circle.contains_point(&Point2D::new(6.0, 0.0))); // 外部
    }

    #[test]
    fn test_point_at_angle() {
        let circle = Circle::new(Point2D::new(0.0, 0.0), 2.0);

        let point = circle.point_at_angle(0.0);
        assert!((point.x() - 2.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);

        let point = circle.point_at_angle(PI / 2.0);
        assert!((point.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((point.y() - 2.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_bounding_box() {
        let circle = Circle::new(Point2D::new(1.0, 2.0), 3.0);
        let (min, max) = circle.bounding_box();

        assert_eq!(min, Point2D::new(-2.0, -1.0));
        assert_eq!(max, Point2D::new(4.0, 5.0));
    }

    #[test]
    fn test_from_three_points() {
        let p1 = Point2D::new(1.0, 0.0);
        let p2 = Point2D::new(0.0, 1.0);
        let p3 = Point2D::new(-1.0, 0.0);

        let circle = Circle::from_three_points(p1, p2, p3).unwrap();

        // 原点中心の半径1の円になるはず
        assert!((circle.center().x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((circle.center().y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((circle.radius() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }
}
