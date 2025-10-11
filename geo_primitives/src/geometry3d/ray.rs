//! Ray3D - ジェネリック3Dレイ（半無限直線）の実装
//!
//! 起点と方向を持つ3次元半無限直線をサポート
//! 新しい設計では、InfiniteLine3Dを継承した制約版として実装されます

use crate::geometry3d::{Direction3D, Point, Vector};
use geo_foundation::abstract_types::geometry::{
    Direction,
    InfiniteLine3D,
    Ray3D as Ray3DTrait,
    // RayOps,              // TODO: 後で有効化
    // RayIntersection,     // TODO: 後で有効化
};
use geo_foundation::{Angle, Scalar};

/// ジェネリック3Dレイ（半無限直線）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D<T: Scalar> {
    /// レイの起点
    origin: Point<T>,
    /// レイの方向ベクトル（正規化済み）
    direction: Direction3D<T>,
}

impl<T: Scalar> Ray3D<T> {
    /// 新しいRay3Dを作成（方向ベクトルは内部で正規化）
    pub fn new(origin: Point<T>, direction: Vector<T>) -> Option<Self> {
        Direction3D::from_vector(direction).map(|normalized_dir| Self {
            origin,
            direction: normalized_dir,
        })
    }

    /// Direction3Dから直接作成
    pub fn from_direction(origin: Point<T>, direction: Direction3D<T>) -> Self {
        Self { origin, direction }
    }

    /// 2点を通るレイを作成（fromからtoward方向）
    pub fn from_two_points(from: Point<T>, toward: Point<T>) -> Option<Self> {
        let direction_vector = toward - from;
        Self::new(from, direction_vector)
    }

    /// X軸正方向のレイを作成
    pub fn x_axis(origin: Point<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_x(),
        }
    }

    /// Y軸正方向のレイを作成
    pub fn y_axis(origin: Point<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_y(),
        }
    }

    /// Z軸正方向のレイを作成
    pub fn z_axis(origin: Point<T>) -> Self {
        Self {
            origin,
            direction: Direction3D::positive_z(),
        }
    }

    /// レイを平行移動
    pub fn translate(&self, vector: &Vector<T>) -> Self {
        Self {
            origin: self.origin + *vector,
            direction: self.direction,
        }
    }

    /// レイをX軸周りで回転
    pub fn rotate_x(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point::new(
            self.origin.x(),
            self.origin.y() * cos_a - self.origin.z() * sin_a,
            self.origin.y() * sin_a + self.origin.z() * cos_a,
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x(),
            dir.y() * cos_a - dir.z() * sin_a,
            dir.y() * sin_a + dir.z() * cos_a,
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをY軸周りで回転
    pub fn rotate_y(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point::new(
            self.origin.x() * cos_a + self.origin.z() * sin_a,
            self.origin.y(),
            -self.origin.x() * sin_a + self.origin.z() * cos_a,
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x() * cos_a + dir.z() * sin_a,
            dir.y(),
            -dir.x() * sin_a + dir.z() * cos_a,
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをZ軸周りで回転
    pub fn rotate_z(&self, angle: Angle<T>) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let rotated_origin = Point::new(
            self.origin.x() * cos_a - self.origin.y() * sin_a,
            self.origin.x() * sin_a + self.origin.y() * cos_a,
            self.origin.z(),
        );

        let dir = self.direction.to_vector();
        let rotated_direction_vec = Vector::new(
            dir.x() * cos_a - dir.y() * sin_a,
            dir.x() * sin_a + dir.y() * cos_a,
            dir.z(),
        );

        Self {
            origin: rotated_origin,
            direction: Direction3D::from_vector(rotated_direction_vec).unwrap(),
        }
    }

    /// レイをオイラー角で回転（X, Y, Z軸の順番）
    pub fn rotate_euler(&self, x_angle: Angle<T>, y_angle: Angle<T>, z_angle: Angle<T>) -> Self {
        self.rotate_x(x_angle).rotate_y(y_angle).rotate_z(z_angle)
    }

    /// レイの方向ベクトルを取得
    pub fn direction_vector(&self) -> Vector<T> {
        self.direction.to_vector()
    }

    /// レイ固有：制約付きの点取得（t >= 0のみ）
    pub fn point_at_parameter_ray(&self, t: T) -> Option<Point<T>> {
        if t >= T::ZERO {
            Some(self.origin + self.direction.to_vector() * t)
        } else {
            None // 半無限直線なので t < 0 は無効
        }
    }

    /// レイ固有：制約付きの点判定（前方のみ）
    pub fn contains_point_ray(&self, point: &Point<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point_unlimited(point);
        if distance <= tolerance {
            let param = self.parameter_at_point_unlimited(point);
            param >= -tolerance // 許容誤差を考慮して前方判定
        } else {
            false
        }
    }

    /// 制約なしの距離計算（InfiniteLineの機能）
    pub fn distance_to_point_unlimited(&self, point: &Point<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();

        // 点からレイへの投影
        let projection_length = to_point.dot(&direction_vector);
        let projection_point = self.origin + direction_vector * projection_length;
        point.distance_to(&projection_point)
    }

    /// 制約なしのパラメータ計算（InfiniteLineの機能）
    pub fn parameter_at_point_unlimited(&self, point: &Point<T>) -> T {
        let to_point = *point - self.origin;
        let direction_vector = self.direction.to_vector();
        to_point.dot(&direction_vector)
    }

    /// レイの起点を取得（InfiniteLineの origin メソッドと互換）
    pub fn origin(&self) -> Point<T> {
        self.origin
    }

    /// レイの方向を取得（InfiniteLineの direction メソッドと互換）
    pub fn direction(&self) -> Direction3D<T> {
        self.direction
    }

    /// 制約なしの点取得（InfiniteLineの point_at_parameter メソッドと互換）
    pub fn point_at_parameter(&self, t: T) -> Point<T> {
        self.origin + self.direction.to_vector() * t
    }
}

// 新しいトレイト実装（最小責務原則による分離）
impl<T: Scalar> InfiniteLine3D<T> for Ray3D<T> {
    type Point = Point<T>;
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
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        let projection = direction_vec * projection_length;
        let perpendicular = to_point - projection;
        perpendicular.length()
    }

    fn closest_point(&self, point: &Self::Point) -> Self::Point {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        let projection_length = to_point.dot(&direction_vec);
        self.origin + direction_vec * projection_length
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        self.point_at_parameter(t)
    }

    fn parameter_at_point(&self, point: &Self::Point) -> T {
        let to_point = *point - self.origin;
        let direction_vec = self.direction.to_vector();
        to_point.dot(&direction_vec)
    }

    fn intersect_line(&self, other: &Self) -> Option<Self::Point> {
        // 3D空間での直線交点計算（スキューライン判定含む）
        let d1 = self.direction.to_vector();
        let d2 = other.direction.to_vector();
        let w0 = self.origin - other.origin;

        let a = d1.dot(&d1);
        let b = d1.dot(&d2);
        let c = d2.dot(&d2);
        let d = d1.dot(&w0);
        let e = d2.dot(&w0);

        let denom = a * c - b * b;
        if denom.abs() < T::TOLERANCE {
            return None; // 平行または同一直線
        }

        let t1 = (b * e - c * d) / denom;
        let t2 = (a * e - b * d) / denom;

        let p1 = self.point_at_parameter(t1);
        let p2 = other.point_at_parameter(t2);

        // 交点が一致するかチェック
        if p1.distance_to(&p2) <= T::TOLERANCE {
            Some(p1)
        } else {
            None // スキューライン
        }
    }

    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool {
        let cross = self
            .direction
            .to_vector()
            .cross(&other.direction.to_vector());
        cross.length() <= tolerance
    }

    fn is_coincident_with(&self, other: &Self, tolerance: T) -> bool {
        if !self.is_parallel_to(other, tolerance) {
            return false;
        }
        let distance = self.distance_to_point(&other.origin);
        distance <= tolerance
    }

    fn is_skew_to(&self, other: &Self, tolerance: T) -> bool {
        !self.is_parallel_to(other, tolerance) && self.intersect_line(other).is_none()
    }

    fn distance_to_line(&self, other: &Self) -> T {
        let d1 = self.direction.to_vector();
        let d2 = other.direction.to_vector();
        let w0 = self.origin - other.origin;

        let cross = d1.cross(&d2);
        let cross_length = cross.length();

        if cross_length < T::TOLERANCE {
            // 平行線の場合
            self.distance_to_point(&other.origin)
        } else {
            // スキューラインの場合
            w0.dot(&cross).abs() / cross_length
        }
    }

    fn closest_points_to_line(&self, other: &Self) -> Option<(Self::Point, Self::Point)> {
        let d1 = self.direction.to_vector();
        let d2 = other.direction.to_vector();
        let w0 = self.origin - other.origin;

        let a = d1.dot(&d1);
        let b = d1.dot(&d2);
        let c = d2.dot(&d2);
        let d = d1.dot(&w0);
        let e = d2.dot(&w0);

        let denom = a * c - b * b;
        if denom.abs() < T::TOLERANCE {
            return None; // 平行線
        }

        let t1 = (b * e - c * d) / denom;
        let t2 = (a * e - b * d) / denom;

        let p1 = self.point_at_parameter(t1);
        let p2 = other.point_at_parameter(t2);

        Some((p1, p2))
    }

    fn intersect_plane(
        &self,
        plane_point: &Self::Point,
        plane_normal: &Self::Vector,
    ) -> Option<Self::Point> {
        let ray_direction = self.direction.to_vector();
        let denominator = ray_direction.dot(plane_normal);

        if denominator.abs() < T::TOLERANCE {
            return None; // 直線と平面が平行
        }

        let to_plane = *plane_point - self.origin;
        let t = to_plane.dot(plane_normal) / denominator;

        Some(self.point_at_parameter(t))
    }

    fn rotate_around_axis(
        &self,
        _axis_point: &Self::Point,
        _axis_direction: &Self::Direction,
        _angle: T,
    ) -> Result<Self, Self::Error> {
        // TODO: Vector3DRotation trait が利用可能になったら実装
        // 起点を軸周りに回転
        // let to_origin = self.origin - *axis_point;
        // let rotated_to_origin = to_origin.rotate_around_axis(&axis_direction.to_vector(), angle);
        // let new_origin = *axis_point + rotated_to_origin;
        //
        // // 方向を軸周りに回転
        // let ray_direction = self.direction.to_vector();
        // let rotated_direction = ray_direction.rotate_around_axis(&axis_direction.to_vector(), angle);
        //
        // match Direction3D::from_vector(rotated_direction) {
        //     Some(new_direction) => Ok(Self::from_direction(new_origin, new_direction)),
        //     None => Err("Failed to normalize rotated direction".to_string()),
        // }

        // 暫定的に元のままを返す
        Ok(*self)
    }
}

impl<T: Scalar> Ray3DTrait<T> for Ray3D<T> {
    // Ray3DはInfiniteLine3Dを継承するので、追加メソッドのみ実装
}

// TODO: RayOps実装は後回し
/*
impl<T: Scalar> RayOps<T> for Ray3D<T> {
    fn point_at(&self, t: T) -> Option<Self::Point> {
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None // レイは前方向のみ
        }
    }

    fn parameter_at_point(&self, point: &Self::Point) -> Option<T> {
        let param = self.parameter_at_point(point);
        if param >= T::ZERO {
            Some(param)
        } else {
            None
        }
    }

    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool {
        if self.contains_point(point, tolerance) {
            let param = self.parameter_at_point(point);
            param >= -tolerance // 許容誤差を考慮
        } else {
            false
        }
    }
}

impl<T: Scalar> RayIntersection<T> for Ray3D<T> {
    fn intersect_ray(&self, other: &Self) -> Option<Self::Point> {
        if let Some(intersection) = self.intersect_line(other) {
            let t1 = self.parameter_at_point(&intersection);
            let t2 = other.parameter_at_point(&intersection);

            if t1 >= T::ZERO && t2 >= T::ZERO {
                Some(intersection)
            } else {
                None // 交点が両方のレイの範囲内にない
            }
        } else {
            None
        }
    }
}
*/

// 型エイリアス（後方互換性確保）
/// f64版の3D Ray（デフォルト）
pub type Ray3DF64 = Ray3D<f64>;

/// f32版の3D Ray（高速演算用）
pub type Ray3DF32 = Ray3D<f32>;
