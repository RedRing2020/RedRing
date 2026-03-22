//! InfiniteLine2D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine2D の拡張実装

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_contracts::default_distance_tolerance;
use geo_contracts::{Angle, Scalar};

// ============================================================================
// Extension Methods Implementation
// ============================================================================

impl<T: Scalar> InfiniteLine2D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// X軸に平行な直線を作成（y = y0）
    pub fn horizontal(y: T) -> Self {
        Self::new(Point2D::new(T::ZERO, y), Vector2D::unit_x())
            .expect("Unit vector should always be valid for InfiniteLine2D")
    }

    /// Y軸に平行な直線を作成（x = x0）
    pub fn vertical(x: T) -> Self {
        Self::new(Point2D::new(x, T::ZERO), Vector2D::unit_y())
            .expect("Unit vector should always be valid for InfiniteLine2D")
    }

    /// 傾きと切片からY軸形式の直線を作成（y = mx + b）
    pub fn from_slope_intercept(slope: T, intercept: T) -> Self {
        let direction = Vector2D::new(T::ONE, slope);
        Self::new(Point2D::new(T::ZERO, intercept), direction)
            .expect("Non-zero direction vector should always be valid for InfiniteLine2D")
    }

    // ========================================================================
    // Advanced Geometric Analysis (Extension)
    // ========================================================================

    /// 傾きを取得（垂直線の場合はNone）
    pub fn slope(&self) -> Option<T> {
        if self.direction_internal().x().abs() <= T::EPSILON {
            None // 垂直線
        } else {
            Some(self.direction_internal().y() / self.direction_internal().x())
        }
    }

    /// Y切片を取得（垂直線の場合はNone）
    pub fn y_intercept(&self) -> Option<T> {
        self.slope()
            .map(|slope| self.point_internal().y() - slope * self.point_internal().x())
    }

    /// X切片を取得（水平線の場合はNone）
    pub fn x_intercept(&self) -> Option<T> {
        if self.direction_internal().y().abs() <= T::EPSILON {
            None // 水平線
        } else {
            // 傾きの逆数を使用
            let inv_slope = self.direction_internal().x() / self.direction_internal().y();
            Some(self.point_internal().x() - inv_slope * self.point_internal().y())
        }
    }

    /// 水平線かどうかを判定
    pub fn is_horizontal(&self, tolerance: T) -> bool {
        self.direction_internal().y().abs() <= tolerance
    }

    /// 垂直線かどうかを判定
    pub fn is_vertical(&self, tolerance: T) -> bool {
        self.direction_internal().x().abs() <= tolerance
    }

    /// X軸との角度を取得（ラジアン）
    pub fn angle(&self) -> T {
        self.direction_internal().angle().to_radians()
    }

    // ========================================================================
    // Advanced Relationship Analysis (Extension)
    // ========================================================================

    /// 直線が平行かを判定（角度許容誤差使用）
    pub fn is_parallel(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_parallel_to(&other.direction_internal())
    }

    /// 直線が平行かを判定（カスタム許容誤差）
    pub fn is_parallel_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        self.direction_internal()
            .angle_to(&other.direction_internal())
            <= tolerance
            || (T::PI
                - self
                    .direction_internal()
                    .angle_to(&other.direction_internal()))
            .abs()
                <= tolerance
    }

    /// 直線が同一かを判定
    pub fn is_coincident(&self, other: &Self) -> bool {
        self.is_parallel(other)
            && self.contains_point(&other.point_internal(), default_distance_tolerance::<T>())
    }

    /// 直線が垂直かを判定（角度許容誤差使用）
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.direction_internal()
            .is_perpendicular_to(&other.direction_internal())
    }

    /// 直線が垂直かを判定（カスタム許容誤差）
    pub fn is_perpendicular_with_tolerance(&self, other: &Self, tolerance: T) -> bool {
        let right_angle = T::PI / (T::ONE + T::ONE);
        (self
            .direction_internal()
            .angle_to(&other.direction_internal())
            - right_angle)
            .abs()
            <= tolerance
    }

    /// 他の直線と同じ直線かを判定
    pub fn is_same_line(&self, other: &Self, tolerance: T) -> bool {
        self.is_parallel(other) && self.contains_point(&other.point_internal(), tolerance)
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// 直線を平行移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self::new(self.point_internal() + offset, *self.direction_internal())
            .expect("Existing direction should always be valid for InfiniteLine2D")
    }

    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self::new(self.point_internal(), *(-self.direction_internal()))
            .expect("Negated direction should always be valid for InfiniteLine2D")
    }

    /// 原点周りの回転
    pub fn rotate_around_origin(&self, angle: Angle<T>) -> Self {
        let radians = angle.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();

        // 点の回転
        let new_x = self.point_internal().x() * cos_a - self.point_internal().y() * sin_a;
        let new_y = self.point_internal().x() * sin_a + self.point_internal().y() * cos_a;
        let new_point = Point2D::new(new_x, new_y);

        // 方向ベクトルの回転
        let dir_x = self.direction_internal().x() * cos_a - self.direction_internal().y() * sin_a;
        let dir_y = self.direction_internal().x() * sin_a + self.direction_internal().y() * cos_a;
        let new_direction = Vector2D::new(dir_x, dir_y);

        Self::new(new_point, new_direction)
            .expect("Rotated direction should always be valid for InfiniteLine2D")
    }

    /// 原点周りの回転（T型ラジアン - 後方互換性）
    pub fn rotate_around_origin_radians(&self, angle: T) -> Self {
        self.rotate_around_origin(Angle::from_radians(angle))
    }

    // ========================================================================
    // Conversion Methods (Extension)
    // ========================================================================

    /// 3次元無限直線に拡張（Z=0平面）
    pub fn to_3d(&self) -> crate::InfiniteLine3D<T> {
        crate::InfiniteLine3D::new(
            self.point_internal().to_3d(),
            self.direction_internal().to_3d(),
        )
        .unwrap()
    }

    /// 3次元無限直線に拡張（指定Z値平面）
    pub fn to_3d_at_z(&self, z: T) -> crate::InfiniteLine3D<T> {
        crate::InfiniteLine3D::new(
            self.point_internal().to_3d_with_z(z),
            self.direction_internal().to_3d(),
        )
        .unwrap()
    }

    // ========================================================================
    // Advanced Helper Methods (Extension)
    // ========================================================================

    /// 直線上の最も近い点を取得（project_pointのエイリアス）
    pub fn closest_point(&self, point: &Point2D<T>) -> Point2D<T> {
        self.project_point(point)
    }

    /// 直線の交点を計算（intersection_with_line エイリアス）
    pub fn intersection_with_line(&self, other: &Self) -> Option<Point2D<T>> {
        self.intersection(other)
    }

    /// 方向を反転（reverse_direction エイリアス）
    pub fn reverse_direction(&self) -> Self {
        self.reverse()
    }
}
