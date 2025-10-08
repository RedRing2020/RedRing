//! 2D空間における無限直線の具体的な実装。点と方向ベクトルで定義される。
//! CAD/CAMシステムで使用される直線の基本的な操作を提供。

use crate::geometry2d::{Direction2D, Point2D, Vector2D};

use geo_foundation::{
    abstract_types::geometry::{
        Direction, InfiniteLine2D as InfiniteLine2DTrait, InfiniteLineAnalysis, InfiniteLineBuilder,
    },
    common::constants::GEOMETRIC_TOLERANCE,
};

/// 2D無限直線
///
/// 基準点と方向ベクトルで定義される無限に延びる直線。
/// 直線の方程式: point = origin + t * direction (t ∈ ℝ)
#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine2D {
    /// 直線上の基準点
    origin: crate::geometry2d::Point2DF64,
    /// 直線の方向（正規化済み）
    direction: Direction2D,
}

impl InfiniteLine2D {
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(origin: crate::geometry2d::Point2DF64, direction: Direction2D) -> Self {
        Self { origin, direction }
    }

    /// 2点を通る無限直線を作成
    pub fn from_two_points(
        point1: crate::geometry2d::Point2DF64,
        point2: crate::geometry2d::Point2DF64,
    ) -> Option<Self> {
        let diff = Vector2D::new(point2.x() - point1.x(), point2.y() - point1.y());

        if diff.length() < GEOMETRIC_TOLERANCE {
            return None; // 同じ点では直線を定義できない
        }

        let direction = Direction2D::from_vector(diff)?;
        Some(Self::new(point1, direction))
    }

    /// X軸に平行な直線を作成
    pub fn horizontal(y: f64) -> Self {
        Self::new(
            Point2D::new(0.0, y),
            Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap(),
        )
    }

    /// Y軸に平行な直線を作成
    pub fn vertical(x: f64) -> Self {
        Self::new(
            Point2D::new(x, 0.0),
            Direction2D::from_vector(Vector2D::new(0.0, 1.0)).unwrap(),
        )
    }

    /// 指定した角度の直線を作成（原点を通る）
    pub fn from_angle(angle_radians: f64) -> Self {
        let direction =
            Direction2D::from_vector(Vector2D::new(angle_radians.cos(), angle_radians.sin()))
                .unwrap();
        Self::new(Point2D::origin(), direction)
    }

    /// 直線の傾きを取得（垂直線の場合はNone）
    pub fn slope(&self) -> Option<f64> {
        let dx = self.direction.x();
        if dx.abs() < GEOMETRIC_TOLERANCE {
            None // 垂直線
        } else {
            Some(self.direction.y() / dx)
        }
    }

    /// 直線の方程式の係数を取得 (ax + by + c = 0)
    pub fn equation_coefficients(&self) -> (f64, f64, f64) {
        let dir = self.direction.to_vector();
        let normal = Vector2D::new(-dir.y(), dir.x()); // 90度回転で法線ベクトル

        let a = normal.x();
        let b = normal.y();
        let c = -(a * self.origin.x() + b * self.origin.y());

        (a, b, c)
    }

    /// Y切片を取得（垂直線の場合はNone）
    pub fn y_intercept(&self) -> Option<f64> {
        self.slope()
            .map(|slope| self.origin.y() - slope * self.origin.x())
    }

    /// X切片を取得（水平線の場合はNone）
    pub fn x_intercept(&self) -> Option<f64> {
        let dy = self.direction.y();
        if dy.abs() < GEOMETRIC_TOLERANCE {
            None // 水平線
        } else {
            Some(self.origin.x() - self.origin.y() * self.direction.x() / dy)
        }
    }
}

impl InfiniteLine2DTrait<f64> for InfiniteLine2D {
    type Point = crate::geometry2d::Point2DF64;
    type Vector = Vector2D;
    type Direction = Direction2D;
    type Error = String;

    fn origin(&self) -> Self::Point {
        self.origin
    }

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn contains_point(&self, point: &Self::Point, tolerance: f64) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> f64 {
        let to_point = Vector2D::new(point.x() - self.origin.x(), point.y() - self.origin.y());
        let dir_vec = self.direction.to_vector();

        // 外積の絶対値が距離に相当
        (to_point.x() * dir_vec.y() - to_point.y() * dir_vec.x()).abs()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = Vector2D::new(point.x() - self.origin.x(), point.y() - self.origin.y());
        let dir_vec = self.direction.to_vector();

        // 投影係数を計算
        let projection = to_point.dot(&dir_vec);

        Point2D::new(
            self.origin.x() + projection * dir_vec.x(),
            self.origin.y() + projection * dir_vec.y(),
        )
    }

    fn point_at_parameter(&self, t: f64) -> Self::Point {
        let dir_vec = self.direction.to_vector();
        Point2D::new(
            self.origin.x() + t * dir_vec.x(),
            self.origin.y() + t * dir_vec.y(),
        )
    }

    fn parameter_at_point(&self, point: &Self::Point) -> f64 {
        let to_point = Vector2D::new(point.x() - self.origin.x(), point.y() - self.origin.y());
        let dir_vec = self.direction.to_vector();

        // 方向ベクトルへの投影
        to_point.dot(&dir_vec)
    }

    fn intersect_line(&self, other: &Self) -> Option<Self::Point> {
        let (a1, b1, c1) = self.equation_coefficients();
        let (a2, b2, c2) = other.equation_coefficients();

        let det = a1 * b2 - a2 * b1;

        if det.abs() < GEOMETRIC_TOLERANCE {
            return None; // 平行または同一直線
        }

        let x = (b1 * c2 - b2 * c1) / det;
        let y = (a2 * c1 - a1 * c2) / det;

        Some(Point2D::new(x, y))
    }

    fn is_parallel_to(&self, other: &Self, tolerance: f64) -> bool {
        let dir1 = self.direction.to_vector();
        let dir2 = other.direction.to_vector();

        // 外積の絶対値が0に近ければ平行
        (dir1.x() * dir2.y() - dir1.y() * dir2.x()).abs() < tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: f64) -> bool {
        // 平行かつ、一方の点がもう一方の直線上にある
        self.is_parallel_to(other, tolerance) && self.distance_to_point(&other.origin) < tolerance
    }

    fn normal_vector(&self) -> Self::Vector {
        let dir = self.direction.to_vector();
        Vector2D::new(-dir.y(), dir.x()) // 90度回転
    }
}

impl InfiniteLineBuilder<f64> for InfiniteLine2D {
    type Point = crate::geometry2d::Point2DF64;
    type Vector = Vector2D;
    type Direction = Direction2D;
    type InfiniteLine = InfiniteLine2D;
    type Error = String;

    fn from_point_and_direction(
        point: Self::Point,
        direction: Self::Direction,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        Ok(InfiniteLine2D::new(point, direction))
    }

    fn from_two_points(
        point1: Self::Point,
        point2: Self::Point,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        InfiniteLine2D::from_two_points(point1, point2)
            .ok_or_else(|| "Cannot create line from identical points".to_string())
    }

    fn parallel_through_point(
        point: Self::Point,
        reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        Ok(InfiniteLine2D::new(point, reference_line.direction))
    }

    fn perpendicular_through_point_2d(
        point: Self::Point,
        reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        let ref_dir = reference_line.direction.to_vector();
        let perp_dir = Direction2D::from_vector(Vector2D::new(-ref_dir.y(), ref_dir.x()))
            .ok_or_else(|| "Failed to create perpendicular direction".to_string())?;

        Ok(InfiniteLine2D::new(point, perp_dir))
    }
}

impl InfiniteLineAnalysis<f64> for InfiniteLine2D {
    type Point = crate::geometry2d::Point2DF64;
    type Vector = Vector2D;

    fn line_equation_2d(&self) -> (f64, f64, f64) {
        self.equation_coefficients()
    }

    fn sample_points(
        &self,
        start_param: f64,
        end_param: f64,
        num_points: usize,
    ) -> Vec<Self::Point> {
        if num_points == 0 {
            return Vec::new();
        }

        if num_points == 1 {
            let t = (start_param + end_param) / 2.0;
            return vec![self.point_at_parameter(t)];
        }

        let mut points = Vec::with_capacity(num_points);
        let step = (end_param - start_param) / (num_points - 1) as f64;

        for i in 0..num_points {
            let t = start_param + i as f64 * step;
            points.push(self.point_at_parameter(t));
        }

        points
    }

    fn clip_to_bounds(
        &self,
        min_point: Self::Point,
        max_point: Self::Point,
    ) -> Option<(Self::Point, Self::Point)> {
        let mut intersections = Vec::new();

        // 境界ボックスの4辺との交点を計算
        let bounds = [
            // 下辺: y = min_point.y
            InfiniteLine2D::horizontal(min_point.y()),
            // 上辺: y = max_point.y
            InfiniteLine2D::horizontal(max_point.y()),
            // 左辺: x = min_point.x
            InfiniteLine2D::vertical(min_point.x()),
            // 右辺: x = max_point.x
            InfiniteLine2D::vertical(max_point.x()),
        ];

        for bound in &bounds {
            if let Some(intersection) = self.intersect_line(bound) {
                // 交点が境界ボックス内にあるかチェック
                if intersection.x() >= min_point.x() - GEOMETRIC_TOLERANCE
                    && intersection.x() <= max_point.x() + GEOMETRIC_TOLERANCE
                    && intersection.y() >= min_point.y() - GEOMETRIC_TOLERANCE
                    && intersection.y() <= max_point.y() + GEOMETRIC_TOLERANCE
                {
                    intersections.push(intersection);
                }
            }
        }

        // 重複除去
        intersections.dedup_by(|a, b| a.distance_to(b) < GEOMETRIC_TOLERANCE);

        if intersections.len() >= 2 {
            Some((intersections[0], intersections[1]))
        } else {
            None
        }
    }

    fn intersects_with(
        &self,
        _other: &dyn InfiniteLineAnalysis<f64, Point = Self::Point, Vector = Self::Vector>,
    ) -> bool {
        // 簡単な実装：常にtrueを返す（詳細な実装は具体的な型が必要）
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_line_creation() {
        let origin = Point2D::new(1.0, 2.0);
        let direction = Direction2D::from_vector(Vector2D::new(1.0, 0.0)).unwrap();
        let line = InfiniteLine2D::new(origin, direction);

        assert_eq!(line.origin(), origin);
        assert_eq!(line.direction(), direction);
    }

    #[test]
    fn test_from_two_points() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(1.0, 1.0);
        let line = InfiniteLine2D::from_two_points(p1, p2).unwrap();

        assert_eq!(line.origin(), p1);
        assert!((line.direction().x() - line.direction().y()).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_horizontal_vertical_lines() {
        let h_line = InfiniteLine2D::horizontal(3.0);
        assert_eq!(h_line.origin().y(), 3.0);
        assert!((h_line.direction().y()).abs() < GEOMETRIC_TOLERANCE);

        let v_line = InfiniteLine2D::vertical(2.0);
        assert_eq!(v_line.origin().x(), 2.0);
        assert!((v_line.direction().x()).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_distance_to_point() {
        let line = InfiniteLine2D::horizontal(0.0); // X軸
        let point = Point2D::new(5.0, 3.0);
        let distance = line.distance_to_point(&point);

        assert!((distance - 3.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_closest_point() {
        let line = InfiniteLine2D::horizontal(0.0); // X軸
        let point = Point2D::new(5.0, 3.0);
        let closest = line.closest_point(&point);

        assert!((closest.x() - 5.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((closest.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_line_intersection() {
        let line1 = InfiniteLine2D::horizontal(0.0); // X軸
        let line2 = InfiniteLine2D::vertical(0.0); // Y軸

        let intersection = line1.intersect_line(&line2).unwrap();
        assert!((intersection.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((intersection.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_parallel_lines() {
        let line1 = InfiniteLine2D::horizontal(0.0);
        let line2 = InfiniteLine2D::horizontal(1.0);

        assert!(line1.is_parallel_to(&line2, GEOMETRIC_TOLERANCE));
        assert!(!line1.is_coincident_with(&line2, GEOMETRIC_TOLERANCE));
    }

    #[test]
    fn test_equation_coefficients() {
        let line = InfiniteLine2D::from_two_points(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0))
            .unwrap();

        let (a, b, c) = line.equation_coefficients();

        // 点(0,0)と(1,1)が方程式を満たすかチェック
        assert!((a * 0.0 + b * 0.0 + c).abs() < GEOMETRIC_TOLERANCE);
        assert!((a * 1.0 + b * 1.0 + c).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_parameter_at_point() {
        let line = InfiniteLine2D::from_two_points(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0))
            .unwrap();

        let point = Point2D::new(5.0, 0.0);
        let param = line.parameter_at_point(&point);

        assert!((param - 5.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_sample_points() {
        let line = InfiniteLine2D::horizontal(0.0);
        let points = line.sample_points(-2.0, 2.0, 5);

        assert_eq!(points.len(), 5);
        assert!((points[0].x() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((points[4].x() - 2.0).abs() < GEOMETRIC_TOLERANCE);

        for point in &points {
            assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        }
    }
}
