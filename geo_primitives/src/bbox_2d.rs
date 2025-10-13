//! 2次元境界ボックス（BBox2D）の Core 実装
//!
//! Core Foundation パターンに基づく BBox2D の必須機能のみ
//! 拡張機能は bbox_2d_extensions.rs を参照

use crate::Point2D;
use geo_foundation::{
    abstract_types::geometry::core_foundation::{BasicContainment, BasicMetrics, CoreFoundation},
    Scalar,
};

/// 2次元軸平行境界ボックス
///
/// 最小点と最大点で定義される矩形領域
/// 軸に平行な辺を持つ境界ボックス
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BBox2D<T: Scalar> {
    min: Point2D<T>,
    max: Point2D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> BBox2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 最小点と最大点から境界ボックスを作成
    pub fn new(min: Point2D<T>, max: Point2D<T>) -> Self {
        // 座標を正規化（min <= max）
        let actual_min = Point2D::new(min.x().min(max.x()), min.y().min(max.y()));
        let actual_max = Point2D::new(min.x().max(max.x()), min.y().max(max.y()));

        Self {
            min: actual_min,
            max: actual_max,
        }
    }

    /// 点から境界ボックスを作成（点の境界ボックス = 点自身）
    pub fn from_point(point: Point2D<T>) -> Self {
        Self::new(point, point)
    }

    /// 複数の点から境界ボックスを作成
    pub fn from_points(points: &[Point2D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut max_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_y = points[0].y();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            max_x = max_x.max(point.x());
            min_y = min_y.min(point.y());
            max_y = max_y.max(point.y());
        }

        Some(Self::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    /// 原点中心の正方形境界ボックスを作成
    pub fn centered_square(half_size: T) -> Self {
        Self::new(
            Point2D::new(-half_size, -half_size),
            Point2D::new(half_size, half_size),
        )
    }

    /// 指定中心と大きさの境界ボックスを作成
    pub fn from_center_size(center: Point2D<T>, width: T, height: T) -> Self {
        let half_width = width / (T::ONE + T::ONE);
        let half_height = height / (T::ONE + T::ONE);

        Self::new(
            Point2D::new(center.x() - half_width, center.y() - half_height),
            Point2D::new(center.x() + half_width, center.y() + half_height),
        )
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 最小点を取得
    pub fn min(&self) -> Point2D<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max(&self) -> Point2D<T> {
        self.max
    }

    /// 境界ボックスの幅を取得
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 境界ボックスの高さを取得
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 境界ボックスの面積を取得
    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    /// 境界ボックスの中心点を取得
    pub fn center(&self) -> Point2D<T> {
        let two = T::ONE + T::ONE;
        Point2D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    /// 境界ボックスのサイズ（幅、高さ）を取得
    pub fn size(&self) -> (T, T) {
        (self.width(), self.height())
    }

    // ========================================================================
    // Core Containment Methods
    // ========================================================================

    /// 点が境界ボックス内に含まれるかを判定
    pub fn contains_point(&self, point: &Point2D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// 他の境界ボックスが完全に含まれるかを判定
    pub fn contains_bbox(&self, other: &Self) -> bool {
        self.contains_point(&other.min) && self.contains_point(&other.max)
    }

    /// 点から境界ボックスの境界への最短距離
    pub fn distance_to_point(&self, point: &Point2D<T>) -> T {
        // 内部にある場合は距離0
        if self.contains_point(point) {
            return T::ZERO;
        }

        // 外部の場合、最も近い境界への距離
        let dx = if point.x() < self.min.x() {
            self.min.x() - point.x()
        } else if point.x() > self.max.x() {
            point.x() - self.max.x()
        } else {
            T::ZERO
        };

        let dy = if point.y() < self.min.y() {
            self.min.y() - point.y()
        } else if point.y() > self.max.y() {
            point.y() - self.max.y()
        } else {
            T::ZERO
        };

        (dx * dx + dy * dy).sqrt()
    }

    // ========================================================================
    // Core Validation Methods
    // ========================================================================

    /// 境界ボックスが有効かどうかを判定
    pub fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y()
    }

    /// 境界ボックスが退化しているか（面積が0またはほぼ0）
    pub fn is_degenerate(&self, tolerance: T) -> bool {
        self.width() <= tolerance || self.height() <= tolerance
    }

    /// 境界ボックスが点かどうか（幅も高さも0）
    pub fn is_point(&self, tolerance: T) -> bool {
        self.width() <= tolerance && self.height() <= tolerance
    }
}

// ============================================================================
// Core Foundation Trait Implementations
// ============================================================================

impl<T: Scalar> CoreFoundation<T> for BBox2D<T> {
    type Point = Point2D<T>;
    type Vector = crate::Vector2D<T>;
    type BBox = BBox2D<T>;

    fn bounding_box(&self) -> Self::BBox {
        *self
    }
}

impl<T: Scalar> BasicMetrics<T> for BBox2D<T> {
    fn length(&self) -> Option<T> {
        // 境界ボックスの場合、周囲長を返す
        let perimeter = (self.width() + self.height()) * (T::ONE + T::ONE);
        Some(perimeter)
    }
}

impl<T: Scalar> BasicContainment<T> for BBox2D<T> {
    fn contains_point(&self, point: &Self::Point) -> bool {
        self.contains_point(point)
    }

    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        // 境界上かどうかの判定
        if !self.contains_point(point) {
            return false;
        }

        // 各辺からの距離をチェック
        let dist_to_left = (point.x() - self.min.x()).abs();
        let dist_to_right = (self.max.x() - point.x()).abs();
        let dist_to_bottom = (point.y() - self.min.y()).abs();
        let dist_to_top = (self.max.y() - point.y()).abs();

        dist_to_left <= tolerance
            || dist_to_right <= tolerance
            || dist_to_bottom <= tolerance
            || dist_to_top <= tolerance
    }

    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to_point(point)
    }
}
