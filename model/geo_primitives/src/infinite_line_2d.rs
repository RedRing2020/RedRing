//! 2次元無限直線（InfiniteLine2D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine2Dの必須機能のみ

use crate::{Direction2D, Point2D, Vector2D};
use geo_contracts::{
    AngularRelation, BasicIntersection, ParallelRelation, PerpendicularRelation, SameLineRelation,
    Scalar,
};

/// 2次元無限直線（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine2D<T: Scalar> {
    pub(crate) point: Point2D<T>,         // 直線上の点
    pub(crate) direction: Direction2D<T>, // 正規化された方向ベクトル
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> InfiniteLine2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================
    /// 点と方向ベクトルから無限直線を作成
    pub fn new(point: Point2D<T>, direction: Vector2D<T>) -> Option<Self> {
        Direction2D::from_vector(direction).map(|dir| Self {
            point,
            direction: dir,
        })
    }

    /// 2点から無限直線を作成
    pub fn from_two_points(p1: Point2D<T>, p2: Point2D<T>) -> Option<Self> {
        let direction = Vector2D::from_points(p1, p2);
        Self::new(p1, direction)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 直線上の点を取得（内部用）
    pub(crate) fn point_internal(&self) -> Point2D<T> {
        self.point
    }

    /// 正規化された方向ベクトルを取得（内部用）
    pub(crate) fn direction_internal(&self) -> Direction2D<T> {
        self.direction
    }

    /// 法線ベクトルを取得（右回り90度回転、内部用）
    pub(crate) fn normal_internal(&self) -> Direction2D<T> {
        Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid")
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// 指定パラメータでの点を取得（point + t * direction）
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        self.point + self.direction * t
    }

    /// 点から直線への最短距離を計算
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        let to_point = Vector2D::from_points(self.point, *point);
        let normal = self.normal_internal();
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
        if self.direction.is_parallel_to(&other.direction) {
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
        if det.abs() <= geo_contracts::default_kernel_numerical_zero_tolerance::<T>() {
            return None; // 平行（実際上はありえない）
        }

        let t1 = dp.cross(&(-self.direction)) / det;
        Some(other.point_at_parameter(t1))
    }

    // ========================================================================
    // Core Helper Methods
    // ========================================================================

    /// 境界ボックスを取得（起点を含む十分大きな範囲）
    pub fn bounding_box(&self) -> geo_core::Aabb2D<T> {
        use geo_core::Point2D;
        // 無限直線なので実用的な大きさの境界ボックスを生成
        let large_value = T::from_f64(1e6);
        let half_range = large_value;
        geo_core::Aabb2D::new(
            Point2D::new(self.point.x() - half_range, self.point.y() - half_range),
            Point2D::new(self.point.x() + half_range, self.point.y() + half_range),
        )
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        // 無限直線なので理論上は (-∞, +∞)
        // 実用的な大きな値を使用
        let large_value = T::from_f64(1e6);
        (-large_value, large_value)
    }

    /// 接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector2D<T> {
        // 直線の接線ベクトルは方向ベクトルと同じ
        *self.direction
    }

    /// 境界上判定（直線では点上判定と同じ）
    pub fn on_boundary(&self, point: &Point2D<T>, tolerance: T) -> bool {
        self.contains_point(point, tolerance)
    }
}

// ============================================================================
// Foundation Pattern Core Traits Implementation
// ============================================================================

use geo_contracts::{
    InfiniteLine2DConstructor, InfiniteLine2DContainment, InfiniteLine2DDistance,
    InfiniteLine2DEvaluation, InfiniteLine2DProjection, InfiniteLine2DProperties,
    InfiniteLine2DTransform,
};

/// InfiniteLine2D Constructor Trait Implementation
impl<T: Scalar> InfiniteLine2DConstructor<T> for InfiniteLine2D<T> {
    fn new(point: (T, T), direction: (T, T)) -> Option<Self> {
        let point_2d = Point2D::new(point.0, point.1);
        let direction_vec = Vector2D::new(direction.0, direction.1);
        Self::new(point_2d, direction_vec)
    }

    fn from_two_points(p1: (T, T), p2: (T, T)) -> Option<Self> {
        let point1 = Point2D::new(p1.0, p1.1);
        let point2 = Point2D::new(p2.0, p2.1);
        Self::from_two_points(point1, point2)
    }

    fn horizontal(y: T) -> Self {
        let point = Point2D::new(T::ZERO, y);
        Self::new(point, Vector2D::unit_x()).unwrap()
    }

    fn vertical(x: T) -> Self {
        let point = Point2D::new(x, T::ZERO);
        Self::new(point, Vector2D::unit_y()).unwrap()
    }

    fn x_axis() -> Self {
        Self::horizontal(T::ZERO)
    }

    fn y_axis() -> Self {
        Self::vertical(T::ZERO)
    }

    fn through_origin(direction: (T, T)) -> Option<Self> {
        let origin = Point2D::new(T::ZERO, T::ZERO);
        let direction_vec = Vector2D::new(direction.0, direction.1);
        Self::new(origin, direction_vec)
    }

    // ========== Phase 2 実装 ==========

    fn from_angle(angle: T) -> Self {
        let direction = Vector2D::new(angle.cos(), angle.sin());
        Self::new(Point2D::origin(), direction).unwrap()
    }

    fn from_point_and_angle(point: (T, T), angle: T) -> Self {
        let point_2d = Point2D::new(point.0, point.1);
        let direction = Vector2D::new(angle.cos(), angle.sin());
        Self::new(point_2d, direction).unwrap()
    }

    fn perpendicular_through(point: (T, T), other: &Self) -> Self {
        let point_2d = Point2D::new(point.0, point.1);
        let other_dir = other.direction_internal();
        // 90度回転して垂直方向を取得: (x, y) -> (-y, x)
        let perp_dir = Vector2D::new(-other_dir.y(), other_dir.x());
        Self::new(point_2d, perp_dir).unwrap()
    }
}

/// InfiniteLine2D Properties Trait Implementation
impl<T: Scalar> InfiniteLine2DProperties<T> for InfiniteLine2D<T> {
    fn point(&self) -> (T, T) {
        (self.point.x(), self.point.y())
    }

    fn direction(&self) -> (T, T) {
        (self.direction.x(), self.direction.y())
    }

    fn normal(&self) -> (T, T) {
        let n = Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid");
        (n.x(), n.y())
    }

    fn slope(&self) -> Option<T> {
        if self.direction.x().abs() <= geo_contracts::default_distance_tolerance::<T>() {
            None // 垂直線
        } else {
            Some(self.direction.y() / self.direction.x())
        }
    }

    fn y_intercept(&self) -> Option<T> {
        self.slope()
            .map(|slope| self.point.y() - slope * self.point.x())
    }

    fn x_intercept(&self) -> Option<T> {
        if self.direction.y().abs() <= geo_contracts::default_distance_tolerance::<T>() {
            None // 水平線
        } else {
            // x = (y - b) / m, y=0のときのx
            if let Some(slope) = self.slope() {
                let b = self.y_intercept().unwrap_or(T::ZERO);
                Some(-b / slope)
            } else {
                Some(self.point.x()) // 垂直線のx座標
            }
        }
    }

    fn is_horizontal(&self) -> bool {
        self.direction.y().abs() <= geo_contracts::default_distance_tolerance::<T>()
    }

    fn is_vertical(&self) -> bool {
        self.direction.x().abs() <= geo_contracts::default_distance_tolerance::<T>()
    }

    fn passes_through_origin(&self) -> bool {
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&Point2D::origin(), default_distance_tolerance::<T>())
    }

    fn dimension(&self) -> u32 {
        2
    }

    // ========== Phase 2 実装 ==========

    fn angle(&self) -> T {
        self.direction.y().atan2(self.direction.x())
    }

    fn is_above(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        let vec_to_point = p - self.point;
        let cross = self.direction.cross(&vec_to_point);
        cross > T::ZERO
    }

    fn is_below(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        !self.is_above(point)
            && !self.contains_point(&p, geo_contracts::default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> InfiniteLine2DEvaluation<T> for InfiniteLine2D<T> {
    fn point_at_parameter(&self, t: T) -> (T, T) {
        let p = self.point_at_parameter(t);
        (p.x(), p.y())
    }

    fn parameter_for_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.parameter_for_point(&p)
    }
}

impl<T: Scalar> InfiniteLine2DDistance<T> for InfiniteLine2D<T> {
    fn distance_to_point(&self, point: (T, T)) -> T {
        let p = Point2D::new(point.0, point.1);
        self.distance_to_point(&p)
    }
}

impl<T: Scalar> InfiniteLine2DContainment<T> for InfiniteLine2D<T> {
    fn contains_point(&self, point: (T, T)) -> bool {
        let p = Point2D::new(point.0, point.1);
        use geo_contracts::default_distance_tolerance;
        self.contains_point(&p, default_distance_tolerance::<T>())
    }
}

impl<T: Scalar> InfiniteLine2DProjection<T> for InfiniteLine2D<T> {
    fn project_point(&self, point: (T, T)) -> (T, T) {
        let p = Point2D::new(point.0, point.1);
        let projected = self.project_point(&p);
        (projected.x(), projected.y())
    }

    // ========== Phase 2 実装 ==========

    fn mirror_point(&self, point: (T, T)) -> (T, T) {
        let p = Point2D::new(point.0, point.1);
        let projected = self.project_point(&p);
        // 鏡面点 = 2 * 投影点 - 元の点
        let mirrored = projected + (projected - p);
        (mirrored.x(), mirrored.y())
    }
}

impl<T: Scalar> InfiniteLine2DTransform<T> for InfiniteLine2D<T> {
    fn reverse(&self) -> Self {
        Self::new(self.point, -(*self.direction)).unwrap()
    }

    fn offset(&self, distance: T) -> Self {
        let normal = Direction2D::from_vector(self.direction.rotate_neg_90())
            .expect("Rotated direction should be valid");
        let offset_point = self.point + normal * distance;
        Self::new(offset_point, *self.direction).unwrap()
    }

    fn rotate_around_origin(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 点を回転
        let rotated_point = Point2D::new(
            self.point.x() * cos_a - self.point.y() * sin_a,
            self.point.x() * sin_a + self.point.y() * cos_a,
        );

        // 方向ベクトルを回転
        let rotated_dir = Vector2D::new(
            self.direction.x() * cos_a - self.direction.y() * sin_a,
            self.direction.x() * sin_a + self.direction.y() * cos_a,
        );

        Self::new(rotated_point, rotated_dir).unwrap()
    }

    fn rotate_around_point(&self, center: (T, T), angle: T) -> Self {
        let center_pt = Point2D::new(center.0, center.1);
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // 中心からの相対位置を計算
        let relative = self.point - center_pt;
        let rotated_relative = Point2D::new(
            relative.x() * cos_a - relative.y() * sin_a,
            relative.x() * sin_a + relative.y() * cos_a,
        );
        let rotated_point = center_pt + (rotated_relative - Point2D::origin());

        // 方向ベクトルを回転
        let rotated_dir = Vector2D::new(
            self.direction.x() * cos_a - self.direction.y() * sin_a,
            self.direction.x() * sin_a + self.direction.y() * cos_a,
        );

        Self::new(rotated_point, rotated_dir).unwrap()
    }
}

impl<T: Scalar> BasicIntersection<T, Self> for InfiniteLine2D<T> {
    type Point = (T, T);

    fn intersection_with(&self, other: &Self, _tolerance: T) -> Option<Self::Point> {
        InfiniteLine2D::intersection(self, other).map(|point| (point.x(), point.y()))
    }
}

impl<T: Scalar> ParallelRelation<Self> for InfiniteLine2D<T> {
    fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
    }
}

impl<T: Scalar> PerpendicularRelation<Self> for InfiniteLine2D<T> {
    fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.direction.is_perpendicular_to(&other.direction)
    }
}

impl<T: Scalar> SameLineRelation<Self> for InfiniteLine2D<T> {
    fn is_same_line(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
            && {
                use geo_contracts::default_distance_tolerance;
                self.contains_point(&other.point, default_distance_tolerance::<T>())
            }
    }
}

impl<T: Scalar> AngularRelation<T, Self> for InfiniteLine2D<T> {
    fn angle_to(&self, other: &Self) -> T {
        let dot = self.direction.dot(&other.direction);
        let clamped = dot.max(-T::ONE).min(T::ONE);
        clamped.acos()
    }
}
