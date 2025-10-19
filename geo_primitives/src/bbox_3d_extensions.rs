//! 3次元境界ボックス（BBox3D）の Extension 実装
//!
//! Core Foundation パターンに基づく BBox3D の拡張機能
//! 高度な幾何計算、交差判定、変換処理等を提供

use crate::{BBox3D, Point3D, Circle3D, Ellipse3D};
use geo_foundation::Scalar;

// ============================================================================
// Extension Implementation (高度な機能)
// ============================================================================

impl<T: Scalar> BBox3D<T> {
    // ========================================================================
    // Shape Builder Methods (形状からの境界ボックス生成)
    // ========================================================================

    /// 3D円から境界ボックスを作成
    pub fn from_circle(circle: &Circle3D<T>) -> Self {
        let center = circle.center();
        let radius = circle.radius();
        
        // 円の法線方向に関係なく、すべての軸方向にradius分拡張
        let min_point = Point3D::new(
            center.x() - radius,
            center.y() - radius,
            center.z() - radius,
        );
        let max_point = Point3D::new(
            center.x() + radius,
            center.y() + radius,
            center.z() + radius,
        );
        
        Self::new(min_point, max_point)
    }

    /// 3D楕円から境界ボックスを作成
    pub fn from_ellipse(ellipse: &Ellipse3D<T>) -> Self {
        let center = ellipse.center();
        let semi_major = ellipse.semi_major_axis();
        let semi_minor = ellipse.semi_minor_axis();
        
        // 簡略化：楕円を包含する立方体として計算
        let max_radius = semi_major.max(semi_minor);
        let min_point = Point3D::new(
            center.x() - max_radius,
            center.y() - max_radius,
            center.z() - max_radius,
        );
        let max_point = Point3D::new(
            center.x() + max_radius,
            center.y() + max_radius,
            center.z() + max_radius,
        );
        
        Self::new(min_point, max_point)
    }

    /// 複数の点から境界ボックスを作成（内部用）
    pub fn from_point_collection(points: &[Point3D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut min_y = points[0].y();
        let mut min_z = points[0].z();
        let mut max_x = points[0].x();
        let mut max_y = points[0].y();
        let mut max_z = points[0].z();

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            min_z = min_z.min(point.z());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
            max_z = max_z.max(point.z());
        }

        Some(Self::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
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
            || other.max().y() < self.min().y()
            || self.max().z() < other.min().z()
            || other.max().z() < self.min().z())
    }

    /// 他の境界ボックスとの交差領域を取得
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }

        let min_x = self.min().x().max(other.min().x());
        let min_y = self.min().y().max(other.min().y());
        let min_z = self.min().z().max(other.min().z());
        let max_x = self.max().x().min(other.max().x());
        let max_y = self.max().y().min(other.max().y());
        let max_z = self.max().z().min(other.max().z());

        Some(Self::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        ))
    }

    /// 他の境界ボックスとの結合領域を取得
    pub fn union(&self, other: &Self) -> Self {
        let min_x = self.min().x().min(other.min().x());
        let min_y = self.min().y().min(other.min().y());
        let min_z = self.min().z().min(other.min().z());
        let max_x = self.max().x().max(other.max().x());
        let max_y = self.max().y().max(other.max().y());
        let max_z = self.max().z().max(other.max().z());

        Self::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }

    /// 境界ボックスが退化しているかを判定
    pub fn is_degenerate(&self) -> bool {
        self.width() == T::ZERO || self.height() == T::ZERO || self.depth() == T::ZERO
    }
}