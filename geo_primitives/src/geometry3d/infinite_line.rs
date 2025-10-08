//! InfiniteLine3D - ジェネリチE��3D無限直線�E実裁E//!
//! 3D空間における無限直線�E具体的な実裁E��点と方向�Eクトルで定義される、E//! CAD/CAMシスチE��で使用されめED直線�E基本皁E��操作を提供、E
use crate::geometry3d::{Direction3D, Point3D, Vector};
use geo_foundation::{
    abstract_types::{
        Scalar,
        geometry::{
            Direction, InfiniteLine3D as InfiniteLine3DTrait, InfiniteLineAnalysis, InfiniteLineBuilder,
        },
    },
    common::constants::GEOMETRIC_TOLERANCE,
};

/// ジェネリチE��3D無限直緁E///
/// 基準点と方向�Eクトルで定義される無限に延びる直線、E/// 直線�E方程弁E point = origin + t * direction (t ∁E℁E
#[derive(Debug, Clone, PartialEq)]
pub struct InfiniteLine3D<T: Scalar> {
    /// 直線上�E基準点
    origin: Point3D<T>,
    /// 直線�E方向（正規化済み�E�E    direction: Direction3D<T>,
}

impl<T: Scalar> InfiniteLine3D<T> {
    /// 点と方向�Eクトルから無限直線を作�E
    pub fn new(origin: Point3D<T>, direction: Direction3D<T>) -> Self {
        Self { origin, direction }
    }

    /// 2点を通る無限直線を作�E
    pub fn from_two_points(point1: Point3D<T>, point2: Point3D<T>) -> Option<Self> {
        let diff = Vector::new(
            point2.x() - point1.x(),
            point2.y() - point1.y(),
            point2.z() - point1.z(),
        );
        let direction = Direction3D::from_vector(diff)?;
        Some(Self::new(point1, direction))
    }

    /// X軸方向�E無限直線を作�E
    pub fn x_axis(origin: Point3D<T>) -> Self {
        Self::new(
            origin,
            Direction3D::positive_x(),
        )
    }

    /// Y軸方向�E無限直線を作�E
    pub fn y_axis(origin: Point3D<T>) -> Self {
        Self::new(
            origin,
            Direction3D::positive_y(),
        )
    }

    /// Z軸方向�E無限直線を作�E
    pub fn z_axis(origin: Point3D<T>) -> Self {
        Self::new(
            origin,
            Direction3D::positive_z(),
        )
    }

    /// X軸に平行な直線を作�E
    pub fn along_x_axis(y: T, z: T) -> Self {
        Self::new(
            Point3D::new(T::ZERO, y, z),
            Direction3D::positive_x(),
        )
    }

    /// Y軸に平行な直線を作�E
    pub fn along_y_axis(x: T, z: T) -> Self {
        Self::new(
            Point3D::new(x, T::ZERO, z),
            Direction3D::positive_y(),
        )
    }

    /// Z軸に平行な直線を作�E
    pub fn along_z_axis(x: T, y: T) -> Self {
        Self::new(
            Point3D::new(x, y, T::ZERO),
            Direction3D::positive_z(),
        )
    }

    /// XY平面に投影した2D直線を取征E    pub fn project_to_xy(&self) -> crate::geometry2d::InfiniteLine2D {
        use crate::geometry2d::{Direction2D, Point2D};

        let origin_2d = Point2D::new(self.origin.x(), self.origin.y());
        let dir_2d = Direction2D::from_vector(crate::geometry2d::Vector2D::new(
            self.direction.x(),
            self.direction.y(),
        ))
        .unwrap_or_else(|| {
            // Z方向�E場合、任意�E方向を選抁E            Direction2D::from_vector(crate::geometry2d::Vector2D::new(1.0, 0.0)).unwrap()
        });

        crate::geometry2d::InfiniteLine2D::new(origin_2d, dir_2d)
    }

    /// 持E��した平面に投影した直線を取征E    /// 平面への投影�E�実裁E�E一時的に無効化！E    pub fn project_to_plane(&self, plane_normal: &Vector<T>, plane_point: &Point3D<T>) -> Option<Self> {
        // 褁E��な投影計算�E後で実裁E        None
    }
}

impl<T: Scalar> InfiniteLine3DTrait<T> for InfiniteLine3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;
    type Error = String;

    fn origin(&self) -> Self::Point {
        self.origin
    }

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let to_point = Vector::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 外積�E大きさが距離に相彁E        let cross_product = to_point.cross(&dir_vec);
        cross_product.norm()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = Vector::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 投影係数を計箁E        let projection = to_point.dot(&dir_vec);

        Point3D::new(
            self.origin.x() + projection * dir_vec.x(),
            self.origin.y() + projection * dir_vec.y(),
            self.origin.z() + projection * dir_vec.z(),
        )
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        let dir_vec = self.direction.to_vector();
        Point3D::new(
            self.origin.x() + t * dir_vec.x(),
            self.origin.y() + t * dir_vec.y(),
            self.origin.z() + t * dir_vec.z(),
        )
    }

    fn parameter_at_point(&self, point: &Self::Point) -> T {
        let to_point = Vector3D::new(
            point.x() - self.origin.x(),
            point.y() - self.origin.y(),
            point.z() - self.origin.z(),
        );
        let dir_vec = self.direction.to_vector();

        // 方向�Eクトルへの投影
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
            return None; // 平行また�E同一直緁E        }

        // スキューライン�E��Eじれの位置�E��E場合�E最近点対の中点を返す
        let w_cross_d2 = w.cross(&d2);
        let t1 = w_cross_d2.dot(&cross_d1_d2) / cross_norm_sq;

        let w_cross_d1 = w.cross(&d1);
        let t2 = w_cross_d1.dot(&cross_d1_d2) / cross_norm_sq;

        let point1 = self.point_at_parameter(t1);
        let point2 = other.point_at_parameter(t2);

        // 2点間�E距離が許容誤差冁E��あれば交点とみなぁE        if point1.distance_to(&point2) < GEOMETRIC_TOLERANCE {
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

        // 外積�E大きさぁEに近けれ�E平衁E        let cross_product = dir1.cross(&dir2);
        cross_product.length() < tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: f64) -> bool {
        // 平行かつ、一方の点がもぁE��方の直線上にある
        self.is_parallel_to(other, tolerance) && self.distance_to_point(&other.origin) < tolerance
    }

    fn is_skew_to(&self, other: &Self, tolerance: f64) -> bool {
        // 平行でなく、交差もしなぁE���Eじれの位置�E�E        !self.is_parallel_to(other, tolerance) && self.intersect_line(other).is_none()
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
            // 平行線�E場吁E            return self.distance_to_point(&p2);
        }

        // スキューラインの場吁E        w.dot(&cross_d1_d2).abs() / cross_norm
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
            return None; // 平行緁E        }

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
            return None; // 直線が平面に平衁E        }

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
        // 簡単な実裁E��軸周り�E回転は褁E��なため、エラーを返す
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
        // 3Dでは2D方程式�E無意味なので、XY平面への投影を使用
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

        // 墁E��ボックスの6面との交点を計箁E        let faces = [
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
                // 交点が墁E��ボックス冁E��あるかチェチE��
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

        // 重褁E��去
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
        // 簡単な実裁E��常にtrueを返す�E�詳細な実裁E�E具体的な型が忁E��E��E        true
    }
}
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
