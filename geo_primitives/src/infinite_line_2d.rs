//! 2次元無限直線（InfiniteLine2D）のCore実装
//!
//! Foundation統一システムに基づくInfiniteLine2Dの必須機能のみ

use crate::{Direction2D, Point2D, Vector2D};
use geo_foundation::Scalar;

/// 2次元無限直線（Core実装）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InfiniteLine2D<T: Scalar> {
    point: Point2D<T>,         // 直線上の点
    direction: Direction2D<T>, // 正規化された方向ベクトル
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

    /// 直線上の点を取得
    pub fn point(&self) -> Point2D<T> {
        self.point
    }

    /// 正規化された方向ベクトルを取得
    pub fn direction(&self) -> Direction2D<T> {
        self.direction
    }

    /// 法線ベクトルを取得（右回り90度回転）
    pub fn normal(&self) -> Direction2D<T> {
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

    // ========================================================================
    // Core Helper Methods
    // ========================================================================

    /// 境界ボックスを取得（起点を含む十分大きな範囲）
    pub fn bounding_box(&self) -> crate::BBox2D<T> {
        // 無限直線なので実用的な大きさの境界ボックスを生成
        let large_value = T::from_f64(1000.0);
        crate::BBox2D::<T>::from_center_size(self.point, large_value, large_value)
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
