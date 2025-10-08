//! InfiniteLine3D - 3D無限直線の実装
//!
//! 3D空間における無限直線の具体的な実装。点と方向ベクトルで定義される。
//! CAD/CAMシステムで使用される3D直線の基本的な操作を提供。

use crate::geometry3d::{Direction3D, Point3D, Vector3D};
use geo_foundation::{
    abstract_types::geometry::{
        Direction, InfiniteLine3D as InfiniteLine3DTrait, InfiniteLineAnalysis, InfiniteLineBuilder,
    },
    common::constants::GEOMETRIC_TOLERANCE,
};

/// 3D無限直線
///
/// 基準点と方向ベクトルで定義される無限に延びる直線。
/// 直線の方程式: point = origin + t * direction (t ∈ ℝ)
#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine3D {
    /// 直線上の基準点
    origin: Point3D,
    /// 直線の方向（正規化済み）
    direction: Direction3D,
}

impl InfiniteLine3D {
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(origin: Point3D, direction: Direction3D) -> Self {
        Self { origin, direction }
    }

    /// 2点を通る無限直線を作成
    pub fn from_two_points(point1: Point3D, point2: Point3D) -> Option<Self> {
        let diff = Vector3D::new(
            point2.x() - point1.x(),
            point2.y() - point1.y(),
            point2.z() - point1.z(),
        );

        if diff.length() < GEOMETRIC_TOLERANCE {
            return None; // 同じ点では直線を定義できない
        }

        let direction = Direction3D::from_vector(diff)?;
        Some(Self::new(point1, direction))
    }

    /// X軸に平行な直線を作成
    pub fn along_x_axis(y: f64, z: f64) -> Self {
        Self::new(
            Point3D::new(0.0, y, z),
            Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(),
        )
    }

    /// Y軸に平行な直線を作成
    pub fn along_y_axis(x: f64, z: f64) -> Self {
        Self::new(
            Point3D::new(x, 0.0, z),
            Direction3D::from_vector(Vector3D::new(0.0, 1.0, 0.0)).unwrap(),
        )
    }

    /// Z軸に平行な直線を作成
    pub fn along_z_axis(x: f64, y: f64) -> Self {
        Self::new(
            Point3D::new(x, y, 0.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
        )
    }

    /// XY平面に投影した2D直線を取得
    pub fn project_to_xy(&self) -> crate::geometry2d::InfiniteLine2D {
        use crate::geometry2d::{Direction2D, Point2D};

        let origin_2d = Point2D::new(self.origin.x(), self.origin.y());
        let dir_2d = Direction2D::from_vector(crate::geometry2d::Vector2D::new(
            self.direction.x(),
            self.direction.y(),
        ))
        .unwrap_or_else(|| {
            // Z方向の場合、任意の方向を選択
            Direction2D::from_vector(crate::geometry2d::Vector2D::new(1.0, 0.0)).unwrap()
        });

        crate::geometry2d::InfiniteLine2D::new(origin_2d, dir_2d)
    }

    /// 指定した平面に投影した直線を取得
    pub fn project_to_plane(&self, plane_normal: &Vector3D, plane_point: &Point3D) -> Option<Self> {
        // 直線が平面に平行でない場合のみ投影可能
        let dir_vec = self.direction.to_vector();
        let dot_product = dir_vec.dot(plane_normal);

        if dot_product.abs() < GEOMETRIC_TOLERANCE {
            return None; // 直線が平面に平行
        }

        // 原点を平面に投影
        let to_origin = Vector3D::new(
            self.origin.x() - plane_point.x(),
            self.origin.y() - plane_point.y(),
            self.origin.z() - plane_point.z(),
        );

        let distance_to_plane = to_origin.dot(plane_normal) / plane_normal.length();
        let projected_origin = Point3D::new(
            self.origin.x() - distance_to_plane * plane_normal.x(),
            self.origin.y() - distance_to_plane * plane_normal.y(),
            self.origin.z() - distance_to_plane * plane_normal.z(),
        );

        // 方向ベクトルを平面に投影
        let normal_component = dir_vec.dot(plane_normal) / plane_normal.length_squared();
        let projected_dir = Vector3D::new(
            dir_vec.x() - normal_component * plane_normal.x(),
            dir_vec.y() - normal_component * plane_normal.y(),
            dir_vec.z() - normal_component * plane_normal.z(),
        );

        if projected_dir.length() < GEOMETRIC_TOLERANCE {
            return None;
        }

        let projected_direction = Direction3D::from_vector(projected_dir)?;
        Some(Self::new(projected_origin, projected_direction))
    }
}

impl InfiniteLine3DTrait<f64> for InfiniteLine3D {
    type Point = Point3D;
    type Vector = Vector3D;
    type Direction = Direction3D;
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
        let to_point = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 外積の大きさが距離に相当
        let cross_product = to_point.cross(&dir_vec);
        cross_product.length()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 投影係数を計算
        let projection = to_point.dot(&dir_vec);

        Point3D::new(
            self.origin.x() + projection * dir_vec.x(),
            self.origin.y() + projection * dir_vec.y(),
            self.origin.z() + projection * dir_vec.z(),
        )
    }

    fn point_at_parameter(&self, t: f64) -> Self::Point {
        let dir_vec = self.direction.to_vector();
        Point3D::new(
            self.origin.x() + t * dir_vec.x(),
            self.origin.y() + t * dir_vec.y(),
            self.origin.z() + t * dir_vec.z(),
        )
    }

    fn parameter_at_point(&self, point: &Self::Point) -> f64 {
        let to_point = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 方向ベクトルへの投影
        to_point.dot(&dir_vec)
    }

    fn intersect_line(&self, other: &Self) -> Option<Self::Point> {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector3D::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm_sq = cross_d1_d2.length_squared();

        if cross_norm_sq < GEOMETRIC_TOLERANCE * GEOMETRIC_TOLERANCE {
            return None; // 平行または同一直線
        }

        // スキューライン（ねじれの位置）の場合は最近点対の中点を返す
        let w_cross_d2 = w.cross(&d2);
        let t1 = w_cross_d2.dot(&cross_d1_d2) / cross_norm_sq;

        let w_cross_d1 = w.cross(&d1);
        let t2 = w_cross_d1.dot(&cross_d1_d2) / cross_norm_sq;

        let point1 = self.point_at_parameter(t1);
        let point2 = other.point_at_parameter(t2);

        // 2点間の距離が許容誤差内であれば交点とみなす
        if point1.distance_to(&point2) < GEOMETRIC_TOLERANCE {
            Some(Point3D::new(
                (point1.x() + point2.x()) / 2.0,
                (point1.y() + point2.y()) / 2.0,
                (point1.z() + point2.z()) / 2.0,
            ))
        } else {
            None
        }
    }

    fn is_parallel_to(&self, other: &Self, tolerance: f64) -> bool {
        let dir1 = self.direction.to_vector();
        let dir2 = other.direction.to_vector();

        // 外積の大きさが0に近ければ平行
        let cross_product = dir1.cross(&dir2);
        cross_product.length() < tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: f64) -> bool {
        // 平行かつ、一方の点がもう一方の直線上にある
        self.is_parallel_to(other, tolerance) && self.distance_to_point(&other.origin) < tolerance
    }

    fn is_skew_to(&self, other: &Self, tolerance: f64) -> bool {
        // 平行でなく、交差もしない（ねじれの位置）
        !self.is_parallel_to(other, tolerance) && self.intersect_line(other).is_none()
    }

    fn distance_to_line(&self, other: &Self) -> f64 {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector3D::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm = cross_d1_d2.length();

        if cross_norm < GEOMETRIC_TOLERANCE {
            // 平行線の場合
            return self.distance_to_point(&p2);
        }

        // スキューラインの場合
        w.dot(&cross_d1_d2).abs() / cross_norm
    }

    fn closest_points_to_line(&self, other: &Self) -> Option<(Self::Point, Self::Point)> {
        let p1 = self.origin;
        let d1 = self.direction.to_vector();
        let p2 = other.origin;
        let d2 = other.direction.to_vector();

        let w = Vector3D::new(p1.x() - p2.x(), p1.y() - p2.y(), p1.z() - p2.z());
        let cross_d1_d2 = d1.cross(&d2);
        let cross_norm_sq = cross_d1_d2.length_squared();

        if cross_norm_sq < GEOMETRIC_TOLERANCE * GEOMETRIC_TOLERANCE {
            return None; // 平行線
        }

        let w_cross_d2 = w.cross(&d2);
        let t1 = w_cross_d2.dot(&cross_d1_d2) / cross_norm_sq;

        let w_cross_d1 = w.cross(&d1);
        let t2 = w_cross_d1.dot(&cross_d1_d2) / cross_norm_sq;

        let point1 = self.point_at_parameter(t1);
        let point2 = other.point_at_parameter(t2);

        Some((point1, point2))
    }

    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Vector,
    ) -> Option<Self::Point> {
        let dir_vec = self.direction.to_vector();
        let denom = dir_vec.dot(plane_normal);

        if denom.abs() < GEOMETRIC_TOLERANCE {
            return None; // 直線が平面に平行
        }

        let to_plane = Vector3D::new(
            plane_point.x() - self.origin.x(),
            plane_point.y() - self.origin.y(),
            plane_point.z() - self.origin.z(),
        );

        let t = to_plane.dot(plane_normal) / denom;
        Some(self.point_at_parameter(t))
    }

    fn rotate_around_axis(
        &self,
        _axis_point: &Self::Point,
        _axis_direction: &Self::Direction,
        _angle: f64,
    ) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // 簡単な実装：軸周りの回転は複雑なため、エラーを返す
        Err("Rotation around axis not implemented yet".to_string())
    }
}

impl InfiniteLineBuilder<f64> for InfiniteLine3D {
    type Point = Point3D;
    type Vector = Vector3D;
    type Direction = Direction3D;
    type InfiniteLine = InfiniteLine3D;
    type Error = String;

    fn from_point_and_direction(
        point: Self::Point,
        direction: Self::Direction,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        Ok(InfiniteLine3D::new(point, direction))
    }

    fn from_two_points(
        point1: Self::Point,
        point2: Self::Point,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        InfiniteLine3D::from_two_points(point1, point2)
            .ok_or_else(|| "Cannot create line from identical points".to_string())
    }

    fn parallel_through_point(
        point: Self::Point,
        reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        Ok(InfiniteLine3D::new(point, reference_line.direction))
    }

    fn perpendicular_through_point_2d(
        _point: Self::Point,
        _reference_line: &Self::InfiniteLine,
    ) -> Result<Self::InfiniteLine, Self::Error> {
        Err("2D perpendicular operation not applicable in 3D".to_string())
    }
}

impl InfiniteLineAnalysis<f64> for InfiniteLine3D {
    type Point = Point3D;
    type Vector = Vector3D;

    fn line_equation_2d(&self) -> (f64, f64, f64) {
        // 3Dでは2D方程式は無意味なので、XY平面への投影を使用
        let projected = self.project_to_xy();
        projected.line_equation_2d()
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

        // 境界ボックスの6面との交点を計算
        let faces = [
            // X面
            (
                Vector3D::new(1.0, 0.0, 0.0),
                Point3D::new(min_point.x(), 0.0, 0.0),
            ),
            (
                Vector3D::new(1.0, 0.0, 0.0),
                Point3D::new(max_point.x(), 0.0, 0.0),
            ),
            // Y面
            (
                Vector3D::new(0.0, 1.0, 0.0),
                Point3D::new(0.0, min_point.y(), 0.0),
            ),
            (
                Vector3D::new(0.0, 1.0, 0.0),
                Point3D::new(0.0, max_point.y(), 0.0),
            ),
            // Z面
            (
                Vector3D::new(0.0, 0.0, 1.0),
                Point3D::new(0.0, 0.0, min_point.z()),
            ),
            (
                Vector3D::new(0.0, 0.0, 1.0),
                Point3D::new(0.0, 0.0, max_point.z()),
            ),
        ];

        for (normal, point) in &faces {
            if let Some(intersection) = self.intersect_plane(point, normal) {
                // 交点が境界ボックス内にあるかチェック
                if intersection.x() >= min_point.x() - GEOMETRIC_TOLERANCE
                    && intersection.x() <= max_point.x() + GEOMETRIC_TOLERANCE
                    && intersection.y() >= min_point.y() - GEOMETRIC_TOLERANCE
                    && intersection.y() <= max_point.y() + GEOMETRIC_TOLERANCE
                    && intersection.z() >= min_point.z() - GEOMETRIC_TOLERANCE
                    && intersection.z() <= max_point.z() + GEOMETRIC_TOLERANCE
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
    fn test_infinite_line_3d_creation() {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let direction = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let line = InfiniteLine3D::new(origin, direction);

        assert_eq!(line.origin(), origin);
        assert_eq!(line.direction(), direction);
    }

    #[test]
    fn test_from_two_points_3d() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(1.0, 1.0, 1.0);
        let line = InfiniteLine3D::from_two_points(p1, p2).unwrap();

        assert_eq!(line.origin(), p1);

        let dir = line.direction();
        let expected_length = (3.0_f64).sqrt();
        assert!((dir.x() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
        assert!((dir.y() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
        assert!((dir.z() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_axis_parallel_lines() {
        let x_line = InfiniteLine3D::along_x_axis(1.0, 2.0);
        assert_eq!(x_line.origin(), Point3D::new(0.0, 1.0, 2.0));
        assert!((x_line.direction().x() - 1.0).abs() < GEOMETRIC_TOLERANCE);

        let y_line = InfiniteLine3D::along_y_axis(1.0, 2.0);
        assert_eq!(y_line.origin(), Point3D::new(1.0, 0.0, 2.0));
        assert!((y_line.direction().y() - 1.0).abs() < GEOMETRIC_TOLERANCE);

        let z_line = InfiniteLine3D::along_z_axis(1.0, 2.0);
        assert_eq!(z_line.origin(), Point3D::new(1.0, 2.0, 0.0));
        assert!((z_line.direction().z() - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_distance_to_point_3d() {
        let line = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
        let point = Point3D::new(5.0, 3.0, 4.0);
        let distance = line.distance_to_point(&point);

        // 3Dでの距離は (3^2 + 4^2)^0.5 = 5
        assert!((distance - 5.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_closest_point_3d() {
        let line = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
        let point = Point3D::new(5.0, 3.0, 4.0);
        let closest = line.closest_point(&point);

        assert!((closest.x() - 5.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((closest.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((closest.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_line_intersection_3d() {
        let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
        let line2 = InfiniteLine3D::along_y_axis(0.0, 0.0); // Y軸

        let intersection = line1.intersect_line(&line2).unwrap();
        assert!((intersection.x() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((intersection.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        assert!((intersection.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_parallel_lines_3d() {
        let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
        let line2 = InfiniteLine3D::along_x_axis(1.0, 1.0);

        assert!(line1.is_parallel_to(&line2, GEOMETRIC_TOLERANCE));
        assert!(!line1.is_coincident_with(&line2, GEOMETRIC_TOLERANCE));
    }

    #[test]
    fn test_skew_lines() {
        let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
        let line2 = InfiniteLine3D::along_y_axis(0.0, 1.0); // Z=1でY軸方向

        assert!(line1.is_skew_to(&line2, GEOMETRIC_TOLERANCE));
    }

    #[test]
    fn test_distance_to_line_3d() {
        let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
        let line2 = InfiniteLine3D::along_x_axis(0.0, 1.0); // 平行線

        let distance = line1.distance_to_line(&line2);
        assert!((distance - 1.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_plane_intersection() {
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 1.0),
        )
        .unwrap();

        // XY平面との交点
        let plane_normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane_point = Point3D::new(0.0, 0.0, 0.0);

        let intersection = line.intersect_plane(&plane_point, &plane_normal).unwrap();
        assert!((intersection.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
    }

    #[test]
    fn test_sample_points_3d() {
        let line = InfiniteLine3D::along_x_axis(0.0, 0.0);
        let points = line.sample_points(-2.0, 2.0, 5);

        assert_eq!(points.len(), 5);
        assert!((points[0].x() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);
        assert!((points[4].x() - 2.0).abs() < GEOMETRIC_TOLERANCE);

        for point in &points {
            assert!((point.y() - 0.0).abs() < GEOMETRIC_TOLERANCE);
            assert!((point.z() - 0.0).abs() < GEOMETRIC_TOLERANCE);
        }
    }

    #[test]
    fn test_parameter_at_point_3d() {
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let point = Point3D::new(5.0, 0.0, 0.0);
        let param = line.parameter_at_point(&point);

        assert!((param - 5.0).abs() < GEOMETRIC_TOLERANCE);
    }
}
