//! 2次元無限直線（InfiniteLine2D）の新実装
//!
//! foundation.rs の基盤トレイトに基づく InfiniteLine2D の実装

use crate::{Point2D, Vector2D};
use geo_foundation::{
    abstract_types::geometry::core_foundation::{
        BasicContainment, BasicDirectional, BasicParametric, CoreFoundation,
    },
    Scalar,
};

/// 2次元無限直線
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine2D<T: Scalar> {
    point: Point2D<T>,      // 直線上の点
    direction: Vector2D<T>, // 正規化された方向ベクトル
}

impl<T: Scalar> InfiniteLine2D<T> {
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(point: Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        direction.try_normalize().map(|normalized_direction| Self {
            point,
            direction: normalized_direction,
        })
    }

    /// 2点から無限直線を作成
    pub fn from_two_points(p1: Point2D<T>, p2: Point2D<T>) -> Option<Self> {
        let direction = Vector2D::from_points(p1, p2);
        Self::new(p1, direction)
    }

    /// X軸に平行な直線を作成（y = y0）
    pub fn horizontal(y: T) -> Self {
        Self {
            point: Point2D::new(T::ZERO, y),
            direction: Vector2D::unit_x(),
        }
    }

    /// Y軸に平行な直線を作成（x = x0）
    pub fn vertical(x: T) -> Self {
        Self {
            point: Point2D::new(x, T::ZERO),
            direction: Vector2D::unit_y(),
        }
    }

    /// 傾きと切片からY軸形式の直線を作成（y = mx + b）
    pub fn from_slope_intercept(slope: T, intercept: T) -> Self {
        let direction = Vector2D::new(T::ONE, slope).normalize();
        Self {
            point: Point2D::new(T::ZERO, intercept),
            direction,
        }
    }

    /// 直線上の点を取得
    pub fn point(&self) -> Point2D<T> {
        self.point
    }

    /// 正規化された方向ベクトルを取得
    pub fn direction(&self) -> Vector2D<T> {
        self.direction
    }

    /// 法線ベクトルを取得（右回り90度回転）
    pub fn normal(&self) -> Vector2D<T> {
        self.direction.rotate_neg_90()
    }

    /// 指定パラメータでの点を取得（point + t * direction）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point + self.direction * t
    }

    /// 点から直線への最短距離を計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let to_point = Vector2D::from_points(self.point, *point);
        let normal = self.normal();
        to_point.dot(&normal).abs()
    }

    /// 点が直線上にあるかを判定
    pub fn contains_point(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.distance_to_point(point) <= tolerance
    }

    /// 点を直線上に投影
    pub fn project_point(&self, point: &Point2D<T>) -> Point2D<T> {
        let to_point = Vector2D::from_points(self.point, *point);
        let projection_length = to_point.dot(&self.direction);
        self.point_at_parameter(projection_length)
    }

    /// 点の直線に対するパラメータを取得
    pub fn parameter_for_point(&self, point: &Point2D<T>) -> T {
        let to_point = Vector2D::from_points(self.point, *point);
        to_point.dot(&self.direction)
    }

    /// 他の直線との交点を計算
    pub fn intersection(&self, other: &Self) -> Option<Point2D<T>> {
        // 平行線の場合は交点なし
        if self.direction.is_parallel(&other.direction, T::EPSILON) {
            return None;
        }

        // 連立方程式を解く
        // P1 + t1 * D1 = P2 + t2 * D2
        // (P1 - P2) = t2 * D2 - t1 * D1
        let dp = Vector2D::from_points(other.point, self.point);

        // クラメルの公式で解く
        // |dp.x  -D1.x|   |D2.x  -D1.x|
        // |dp.y  -D1.y| / |D2.y  -D1.y|
        let det = other.direction.cross(&(-self.direction));
        if det.abs() <= T::EPSILON {
            return None; // 平行（実際上はありえない）
        }

        let t1 = dp.cross(&(-self.direction)) / det;
        Some(other.point_at_parameter(t1))
    }

    /// 直線が平行かを判定（角度許容誤差使用）
    pub fn is_parallel(&self, other: &Self) -> bool {
        self.direction
            .is_parallel(&other.direction, T::ANGLE_TOLERANCE)
    }

    /// 直線が平行かを判定（カスタム許容誤差）
    pub fn is_parallel_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.direction.is_parallel(&other.direction, tolerance)
    }

    /// 直線が同一かを判定
    pub fn is_coincident(&self, other: &Self) -> bool {
        self.is_parallel(other) && self.contains_point(&other.point, T::TOLERANCE)
    }

    /// 直線が垂直かを判定（角度許容誤差使用）
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.direction
            .is_perpendicular(&other.direction, T::ANGLE_TOLERANCE)
    }

    /// 直線が垂直かを判定（カスタム許容誤差）
    pub fn is_perpendicular_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.direction.is_perpendicular(&other.direction, tolerance)
    }

    /// 直線上の最も近い点を取得
    pub fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        self.project_point(point)
    }

    /// 直線を平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self {
            point: self.point + offset,
            direction: self.direction,
        }
    }

    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self {
            point: self.point,
            direction: -self.direction,
        }
    }

    /// X軸との角度を取得（ラジアン）
    pub fn angle(&self) -> T {
        self.direction.angle()
    }

    /// 傾きを取得（垂直線の場合はNone）
    pub fn slope(&self) -> Option<T> {
        if self.direction.x().abs() <= T::EPSILON {
            None // 垂直線
        } else {
            Some(self.direction.y() / self.direction.x())
        }
    }

    /// Y切片を取得（垂直線の場合はNone）
    pub fn y_intercept(&self) -> Option<T> {
        self.slope()
            .map(|slope| self.point.y() - slope * self.point.x())
    }

    /// X切片を取得（水平線の場合はNone）
    pub fn x_intercept(&self) -> Option<T> {
        if self.direction.y().abs() <= T::EPSILON {
            None // 水平線
        } else {
            // 傾きの逆数を使用
            let inv_slope = self.direction.x() / self.direction.y();
            Some(self.point.x() - inv_slope * self.point.y())
        }
    }

    /// 水平線かどうかを判定
    pub fn is_horizontal(&self, tolerance: T) -> bool {
        self.direction.y().abs() <= tolerance
    }

    /// 垂直線かどうかを判定
    pub fn is_vertical(&self, tolerance: T) -> bool {
        self.direction.x().abs() <= tolerance
    }

    /// 他の直線と同じ直線かを判定
    pub fn is_same_line(&self, other: &Self, tolerance: T) -> bool {
        self.is_parallel(other) && self.contains_point(&other.point, tolerance)
    }

    /// 直線の交点を計算（intersection_with_line エイリアス）
    pub fn intersection_with_line(&self, other: &Self) -> Option<Point2D<T>> {
        self.intersection(other)
    }

    /// 原点周りの回転
    pub fn rotate_around_origin(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 点の回転
        let new_x = self.point.x() * cos_a - self.point.y() * sin_a;
        let new_y = self.point.x() * sin_a + self.point.y() * cos_a;
        let new_point = Point2D::new(new_x, new_y);

        // 方向ベクトルの回転
        let dir_x = self.direction.x() * cos_a - self.direction.y() * sin_a;
        let dir_y = self.direction.x() * sin_a + self.direction.y() * cos_a;
        let new_direction = Vector2D::new(dir_x, dir_y);

        Self {
            point: new_point,
            direction: new_direction,
        }
    }

    /// 境界ボックスを取得（無限大の範囲として表現）
    pub fn bounding_box(&self) -> crate::BBox2D<T> {
        // 無限直線の境界ボックスは理論上無限大
        // 実用的な大きな値を使用
        let large_value = T::from_f64(1e6);
        crate::BBox2D::new(
            Point2D::new(-large_value, -large_value),
            Point2D::new(large_value, large_value),
        )
    }

    /// 3次元無限直線に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::InfiniteLine3D<T> {
        crate::InfiniteLine3D::new(self.point.to_3d(), self.direction.to_3d()).unwrap()
    }

    /// 3次元無限直線に拡張（指定Z値平面）
    pub fn to_3d_at_z(&self, z: T) -> crate::InfiniteLine3D<T> {
        crate::InfiniteLine3D::new(self.point.to_3d_with_z(z), self.direction.to_3d()).unwrap()
    }
}

// ============================================================================
// Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for InfiniteLine2D<T> {
    type Point = Point2D<T>;
    type Vector = Vector2D<T>;
    type BBox = crate::BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        self.bounding_box()
    }
}

impl<T: Scalar> BasicContainment<T> for InfiniteLine2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point, T::EPSILON)
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        InfiniteLine2D::distance_to_point(self, point)
    }
}

impl<T: Scalar> BasicDirectional<T> for InfiniteLine2D<T> {
    type Direction = Vector2D<T>;

    fn direction(&self) -> Self::Direction {
        self.direction
    }

    fn reverse_direction(&self) -> Self {
        self.reverse()
    }
}

impl<T: Scalar> BasicParametric<T> for InfiniteLine2D<T> {
    fn parameter_range(&self) -> (T, T) {
        // 無限直線なので理論上は (-∞, +∞)
        // 実用的な大きな値を使用
        let large_value = T::from_f64(1e6);
        (-large_value, large_value)
    }

    fn point_at_parameter(&self, t: T) -> Self::Point {
        InfiniteLine2D::point_at_parameter(self, t)
    }

    fn tangent_at_parameter(&self, _t: T) -> Self::Vector {
        // 直線の接線ベクトルは方向ベクトルと同じ
        self.direction
    }
}
