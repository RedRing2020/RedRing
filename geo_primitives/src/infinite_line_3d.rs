//! 無限直線（InfiniteLine）の新実装
//!
//! foundation.rs の基盤トレイトに基づく InfiniteLine3D の実装

use crate::{BBox3D, Point3D, Vector3D};
use geo_foundation::{abstract_types::geometry::foundation::*, Scalar};

/// 3次元空間の無限直線
///
/// 点と方向ベクトルで定義される無限に延びる直線
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine3D<T: Scalar> {
    point: Point3D<T>,      // 直線上の任意の点
    direction: Vector3D<T>, // 方向ベクトル（正規化済み）
}

impl<T: Scalar> InfiniteLine3D<T> {
    /// 新しい無限直線を作成
    ///
    /// # 引数
    /// * `point` - 直線上の任意の点
    /// * `direction` - 方向ベクトル（自動的に正規化される）
    ///
    /// # 戻り値
    /// * `Some(InfiniteLine3D)` - 有効な直線が作成できた場合
    /// * `None` - 方向ベクトルがゼロベクトルの場合
    pub fn new(point: Point3D<T>, direction: Vector3D<T>) -> Option<Self> {
        let normalized_direction = direction.normalize()?;

        Some(Self {
            point,
            direction: normalized_direction,
        })
    }

    /// 2点を通る直線を作成
    pub fn from_two_points(p1: Point3D<T>, p2: Point3D<T>) -> Option<Self> {
        let direction = Vector3D::from_points(&p1, &p2);
        Self::new(p1, direction)
    }

    /// X軸に平行な直線を作成
    pub fn x_axis_line(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_x()).unwrap()
    }

    /// Y軸に平行な直線を作成
    pub fn y_axis_line(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_y()).unwrap()
    }

    /// Z軸に平行な直線を作成
    pub fn z_axis_line(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_z()).unwrap()
    }

    /// 直線上の点を取得
    pub fn point(&self) -> Point3D<T> {
        self.point
    }

    /// 方向ベクトルを取得（正規化済み）
    pub fn direction(&self) -> Vector3D<T> {
        self.direction
    }

    /// パラメータtでの直線上の点を取得
    /// 点 = point + t * direction
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        Point3D::new(
            self.point.x() + t * self.direction.x(),
            self.point.y() + t * self.direction.y(),
            self.point.z() + t * self.direction.z(),
        )
    }

    /// 点を直線に投影
    pub fn project_point(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = Vector3D::from_points(&self.point, point);
        let t = to_point.dot(&self.direction);
        self.point_at_parameter(t)
    }

    /// 点から直線への最短距離
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let projected = self.project_point(point);
        point.distance_to(&projected)
    }

    /// 点が直線上にあるかを判定
    pub fn contains_point(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 点を直線に投影した時のパラメータtを取得
    pub fn parameter_for_point(&self, point: &Point3D<T>) -> T {
        let to_point = Vector3D::from_points(&self.point, point);
        to_point.dot(&self.direction)
    }

    /// 他の直線と平行かどうかを判定
    pub fn is_parallel(&self, other: &Self, _tolerance: T) -> bool {
        self.direction.is_parallel(&other.direction)
            || self.direction.is_parallel(&other.direction.negate())
    }

    /// 他の直線と同一直線かどうかを判定
    pub fn is_coincident(&self, other: &Self, tolerance: T) -> bool {
        self.is_parallel(other, tolerance) && self.contains_point(&other.point, tolerance)
    }

    /// 他の直線との交点を計算（3D空間では一般的に交わらない）
    /// 交わる場合、最も近い点同士を返す
    pub fn closest_points_to_line(&self, other: &Self) -> (Point3D<T>, Point3D<T>) {
        let w = Vector3D::from_points(&other.point, &self.point);
        let d1 = self.direction;
        let d2 = other.direction;

        let d1_dot_d1 = d1.dot(&d1);
        let d1_dot_d2 = d1.dot(&d2);
        let d2_dot_d2 = d2.dot(&d2);
        let w_dot_d1 = w.dot(&d1);
        let w_dot_d2 = w.dot(&d2);

        let denominator = d1_dot_d1 * d2_dot_d2 - d1_dot_d2 * d1_dot_d2;

        let (t1, t2) = if denominator.abs() < T::EPSILON {
            // 平行な直線
            (T::ZERO, w_dot_d2 / d2_dot_d2)
        } else {
            let t1 = (d1_dot_d2 * w_dot_d2 - d2_dot_d2 * w_dot_d1) / denominator;
            let t2 = (d1_dot_d1 * w_dot_d2 - d1_dot_d2 * w_dot_d1) / denominator;
            (t1, t2)
        };

        (self.point_at_parameter(t1), other.point_at_parameter(t2))
    }

    /// 他の直線との最短距離
    pub fn distance_to_line(&self, other: &Self) -> T {
        let (p1, p2) = self.closest_points_to_line(other);
        p1.distance_to(&p2)
    }

    /// 直線を指定ベクトルで平行移動
    pub fn translate(&self, vector: &Vector3D<T>) -> Self {
        let new_point = Point3D::new(
            self.point.x() + vector.x(),
            self.point.y() + vector.y(),
            self.point.z() + vector.z(),
        );
        Self::new(new_point, self.direction).unwrap()
    }

    /// 方向を反転した直線を取得
    pub fn reverse(&self) -> Self {
        Self::new(self.point, self.direction.negate()).unwrap()
    }

    /// 直線の方向ベクトルの各成分が主要軸かどうかを判定
    pub fn is_axis_aligned(&self, tolerance: T) -> bool {
        let d = self.direction;
        let x_aligned = (d.y().abs() < tolerance) && (d.z().abs() < tolerance);
        let y_aligned = (d.x().abs() < tolerance) && (d.z().abs() < tolerance);
        let z_aligned = (d.x().abs() < tolerance) && (d.y().abs() < tolerance);

        x_aligned || y_aligned || z_aligned
    }

    /// どの軸に平行かを判定
    pub fn aligned_axis(&self, tolerance: T) -> Option<Vector3D<T>> {
        let d = self.direction;

        if (d.y().abs() < tolerance) && (d.z().abs() < tolerance) {
            Some(Vector3D::unit_x())
        } else if (d.x().abs() < tolerance) && (d.z().abs() < tolerance) {
            Some(Vector3D::unit_y())
        } else if (d.x().abs() < tolerance) && (d.y().abs() < tolerance) {
            Some(Vector3D::unit_z())
        } else {
            None
        }
    }
}

// === foundation トレイト実装 ===

impl<T: Scalar> GeometryFoundation<T> for InfiniteLine3D<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type BBox = BBox3D<T>;

    /// 無限直線の境界ボックス（無限大）
    /// 実装上は非常に大きな境界ボックスを返す
    fn bounding_box(&self) -> Self::BBox {
        let large_value = T::from_f64(1e10); // 十分大きな値

        BBox3D::new(
            Point3D::new(-large_value, -large_value, -large_value),
            Point3D::new(large_value, large_value, large_value),
        )
    }
}

impl<T: Scalar> BasicContainment<T> for InfiniteLine3D<T> {
    /// 点が直線上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point, T::TOLERANCE)
    }

    /// 点が直線の境界上にあるかを判定（無限直線では contains_point と同じ）
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    /// 点から直線への最短距離
    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to_point(point)
    }
}

impl<T: Scalar> BasicDirectional<T> for InfiniteLine3D<T> {
    type Direction = Vector3D<T>;

    /// 直線の方向ベクトル
    fn direction(&self) -> Self::Direction {
        self.direction
    }

    /// 方向を反転した直線
    fn reverse_direction(&self) -> Self {
        self.reverse()
    }
}

// 無限直線は BasicParametric を実装しない
// パラメータ範囲が無限大のため BasicParametric の概念に適さない
// 代わりに独自の point_at_parameter メソッドを提供
