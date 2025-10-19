//! 2次元境界ボックス（BBox2D）の Extension 実装
//!
//! Core Foundation パターンに基づく BBox2D の拡張機能
//! 高度な幾何計算、交差判定、変換処理等を提供

use crate::{BBox2D, Point2D, Vector2D, Circle2D, Ellipse2D, Arc2D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Extension Implementation (高度な機能)
// ============================================================================

impl<T: Scalar> BBox2D<T> {
    // ========================================================================
    // Shape Builder Methods (形状からの境界ボックス生成)
    // ========================================================================

    /// 円から境界ボックスを作成
    pub fn from_circle(circle: &Circle2D<T>) -> Self {
        let (min_point, max_point) = circle.bounding_box();
        Self::new(min_point, max_point)
    }

    /// 楕円から境界ボックスを作成
    pub fn from_ellipse(ellipse: &Ellipse2D<T>) -> Self {
        ellipse.bounding_box()
    }

    /// 円弧から境界ボックスを作成（サンプリングベース）
    pub fn from_arc(arc: &Arc2D<T>) -> Self {
        // 円弧をサンプリングして境界ボックスを計算
        let num_samples = 16;
        let mut points = Vec::new();
        
        // 開始点と終了点を追加
        points.push(arc.start_point());
        points.push(arc.end_point());
        
        // 中間サンプル点を追加
        let start_rad = arc.start_angle().to_radians();
        let end_rad = arc.end_angle().to_radians();
        let angle_range = end_rad - start_rad;
        
        for i in 1..num_samples {
            let t = T::from_f64(i as f64 / num_samples as f64);
            let angle_rad = start_rad + angle_range * t;
            points.push(arc.point_at_angle(angle_rad));
        }
        
        Self::from_point_collection(&points).unwrap()
    }

    /// 複数の点から境界ボックスを作成
    pub fn from_point_collection(points: &[Point2D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_x = points[0].x();
        let mut max_y = points[0].y();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }

        Some(Self::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    // ========================================================================
    // Advanced Intersection Methods
    // ========================================================================

    /// 他の境界ボックスと交差するかを判定
    pub fn intersects(&self, other: &Self) -> bool {
        !(self.max().x() < other.min().x()
            || other.max().x() < self.min().x()
            || self.max().y() < other.min().y()
            || other.max().y() < self.min().y())
    }

    /// 他の境界ボックスとの交差領域を取得
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let min_x = self.min().x().max(other.min().x());
        let min_y = self.min().y().max(other.min().y());
        let max_x = self.max().x().min(other.max().x());
        let max_y = self.max().y().min(other.max().y());

        Some(Self::new(
            Point2D::new(min_x, min_y),
            Point2D::new(max_x, max_y),
        ))
    }

    /// 他の境界ボックスとの結合領域を取得
    pub fn union(&self, other: &Self) -> Self {
        let min_x = self.min().x().min(other.min().x());
        let min_y = self.min().y().min(other.min().y());
        let max_x = self.max().x().max(other.max().x());
        let max_y = self.max().y().max(other.max().y());

        Self::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 複数の境界ボックスとの結合
    pub fn union_multiple(bboxes: &[Self]) -> Option<Self> {
        if bboxes.is_empty() {
            return None;
        }

        let mut result = bboxes[0];
        for bbox in bboxes.iter().skip(1) {
            result = result.union(bbox);
        }
        Some(result)
    }

    // ========================================================================
    // Advanced Transformation Methods
    // ========================================================================

    /// 境界ボックスを指定マージンで拡張
    pub fn expand(&self, margin: T) -> Self {
        Self::new(
            Point2D::new(self.min().x() - margin, self.min().y() - margin),
            Point2D::new(self.max().x() + margin, self.max().y() + margin),
        )
    }

    /// 境界ボックスを異なるマージンで拡張（X, Y別々）
    pub fn expand_by(&self, margin_x: T, margin_y: T) -> Self {
        Self::new(
            Point2D::new(self.min().x() - margin_x, self.min().y() - margin_y),
            Point2D::new(self.max().x() + margin_x, self.max().y() + margin_y),
        )
    }

    /// 境界ボックスを縮小
    pub fn shrink(&self, margin: T) -> Option<Self> {
        let new_min = Point2D::new(self.min().x() + margin, self.min().y() + margin);
        let new_max = Point2D::new(self.max().x() - margin, self.max().y() - margin);

        // 縮小後も有効な境界ボックスか確認
        if new_min.x() <= new_max.x() && new_min.y() <= new_max.y() {
            Some(Self::new(new_min, new_max))
        } else {
            None
        }
    }

    /// 境界ボックスを指定比率でスケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        let center = self.center();
        let half_width = self.width() * factor / (T::ONE + T::ONE);
        let half_height = self.height() * factor / (T::ONE + T::ONE);

        Some(Self::new(
            Point2D::new(center.x() - half_width, center.y() - half_height),
            Point2D::new(center.x() + half_width, center.y() + half_height),
        ))
    }

    /// 境界ボックスを異なる比率でスケール（X, Y別々）
    pub fn scale_by(&self, factor_x: T, factor_y: T) -> Option<Self> {
        if factor_x <= T::ZERO || factor_y <= T::ZERO {
            return None;
        }

        let center = self.center();
        let half_width = self.width() * factor_x / (T::ONE + T::ONE);
        let half_height = self.height() * factor_y / (T::ONE + T::ONE);

        Some(Self::new(
            Point2D::new(center.x() - half_width, center.y() - half_height),
            Point2D::new(center.x() + half_width, center.y() + half_height),
        ))
    }

    /// 境界ボックスを平行移動
    pub fn translate(&self, offset: &Vector2D<T>) -> Self {
        Self::new(self.min() + *offset, self.max() + *offset)
    }

    /// 境界ボックスを指定点周りで回転
    pub fn rotate_around_point(&self, center: &Point2D<T>, angle: Angle<T>) -> Self {
        // 境界ボックスの回転は角点を回転して新しい境界ボックスを作成
        let corners = self.corners();
        let rotated_corners: Vec<Point2D<T>> = corners
            .iter()
            .map(|corner| corner.rotate_around(center, angle))
            .collect();

        Self::from_point_collection(&rotated_corners).unwrap_or(*self)
    }

    /// 境界ボックスを指定点基準でスケール
    pub fn scale_from_point(&self, center: &Point2D<T>, factor: T) -> Option<Self> {
        if factor <= T::ZERO {
            return None;
        }

        // 手動で中心点基準のスケール計算
        let min_offset = self.min() - *center;
        let max_offset = self.max() - *center;

        let scaled_min = *center + min_offset * factor;
        let scaled_max = *center + max_offset * factor;

        Some(Self::new(scaled_min, scaled_max))
    }

    // ========================================================================
    // Advanced Geometric Queries
    // ========================================================================

    /// 境界ボックスの角の点を取得
    pub fn corners(&self) -> [Point2D<T>; 4] {
        [
            self.min(),                                   // 左下
            Point2D::new(self.max().x(), self.min().y()), // 右下
            self.max(),                                   // 右上
            Point2D::new(self.min().x(), self.max().y()), // 左上
        ]
    }

    /// 境界ボックスの辺の中点を取得
    pub fn edge_midpoints(&self) -> [Point2D<T>; 4] {
        let center = self.center();
        [
            Point2D::new(center.x(), self.min().y()), // 下辺中点
            Point2D::new(self.max().x(), center.y()), // 右辺中点
            Point2D::new(center.x(), self.max().y()), // 上辺中点
            Point2D::new(self.min().x(), center.y()), // 左辺中点
        ]
    }

    /// 境界ボックスの対角線の長さ
    pub fn diagonal_length(&self) -> T {
        let dx = self.width();
        let dy = self.height();
        (dx * dx + dy * dy).sqrt()
    }

    /// 境界ボックスのアスペクト比（幅/高さ）
    pub fn aspect_ratio(&self) -> Option<T> {
        if self.height() == T::ZERO {
            None
        } else {
            Some(self.width() / self.height())
        }
    }

    /// 2つの境界ボックス間の重複面積
    pub fn overlap_area(&self, other: &Self) -> T {
        if let Some(intersection) = self.intersection(other) {
            intersection.area()
        } else {
            T::ZERO
        }
    }

    /// 2つの境界ボックス間の重複率（0〜1）
    pub fn overlap_ratio(&self, other: &Self) -> T {
        let overlap = self.overlap_area(other);
        let total_area = self.area() + other.area() - overlap;

        if total_area == T::ZERO {
            T::ZERO
        } else {
            overlap / total_area
        }
    }

    /// 他の境界ボックスとの距離
    pub fn distance_to_bbox(&self, other: &Self) -> T {
        if self.intersects(other) {
            T::ZERO
        } else {
            // 最も近い角同士の距離を計算
            let self_corners = self.corners();
            let other_corners = other.corners();

            let mut min_distance = self_corners[0].distance_to(&other_corners[0]);
            for self_corner in &self_corners {
                for other_corner in &other_corners {
                    let dist = self_corner.distance_to(other_corner);
                    if dist < min_distance {
                        min_distance = dist;
                    }
                }
            }
            min_distance
        }
    }

    // ========================================================================
    // 3D Conversion Methods
    // ========================================================================

    /// 3次元境界ボックスに変換（Z=0）
    pub fn to_3d(&self) -> crate::BBox3D<T> {
        crate::BBox3D::new(self.min().to_3d(), self.max().to_3d())
    }

    /// 3次元境界ボックスに変換（指定Z範囲）
    pub fn to_3d_with_z(&self, min_z: T, max_z: T) -> crate::BBox3D<T> {
        crate::BBox3D::new(
            self.min().to_3d_with_z(min_z),
            self.max().to_3d_with_z(max_z),
        )
    }

    // ========================================================================
    // Subdivision Methods
    // ========================================================================

    /// 境界ボックスを4分割
    pub fn subdivide(&self) -> [Self; 4] {
        let center = self.center();
        let min = self.min();
        let max = self.max();

        [
            Self::new(min, center), // 左下
            Self::new(
                Point2D::new(center.x(), min.y()),
                Point2D::new(max.x(), center.y()),
            ), // 右下
            Self::new(center, max), // 右上
            Self::new(
                Point2D::new(min.x(), center.y()),
                Point2D::new(center.x(), max.y()),
            ), // 左上
        ]
    }

    /// 境界ボックスを指定数で分割（グリッド）
    pub fn subdivide_grid(&self, rows: usize, cols: usize) -> Vec<Self> {
        if rows == 0 || cols == 0 {
            return vec![];
        }

        let mut result = Vec::with_capacity(rows * cols);

        // from_usizeによる変換
        let cols_t = T::from_usize(cols);
        let rows_t = T::from_usize(rows);

        let cell_width = self.width() / cols_t;
        let cell_height = self.height() / rows_t;

        for row in 0..rows {
            for col in 0..cols {
                let col_t = T::from_usize(col);
                let row_t = T::from_usize(row);

                let min_x = self.min().x() + col_t * cell_width;
                let min_y = self.min().y() + row_t * cell_height;
                let max_x = min_x + cell_width;
                let max_y = min_y + cell_height;

                result.push(Self::new(
                    Point2D::new(min_x, min_y),
                    Point2D::new(max_x, max_y),
                ));
            }
        }

        result
    }
}
