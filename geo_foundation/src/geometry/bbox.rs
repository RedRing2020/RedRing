//! BoundingBox - Scalar traitベースの境界ボックス実装
//!
//! f32/f64両対応の汎用2D/3D境界ボックス実装

use crate::abstract_types::Scalar;
use crate::geometry::{Point2D, Point3D, Vector2D, Vector3D};
use std::fmt::{Debug, Display};

/// 2次元境界ボックス（軸平行境界ボックス - AABB）
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundingBox2D<T: Scalar> {
    min: Point2D<T>,
    max: Point2D<T>,
}

impl<T: Scalar> BoundingBox2D<T> {
    /// 新しい2D境界ボックスを作成
    ///
    /// # Examples
    /// ```
    /// use geo_foundation::BoundingBox2D;
    /// use geo_foundation::Point2D;
    /// let bbox = BoundingBox2D::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 5.0));
    /// ```
    pub fn new(min: Point2D<T>, max: Point2D<T>) -> Self {
        // min/maxの正規化
        let actual_min = Point2D::new(
            if min.x() <= max.x() { min.x() } else { max.x() },
            if min.y() <= max.y() { min.y() } else { max.y() },
        );
        let actual_max = Point2D::new(
            if min.x() >= max.x() { min.x() } else { max.x() },
            if min.y() >= max.y() { min.y() } else { max.y() },
        );
        Self {
            min: actual_min,
            max: actual_max,
        }
    }

    /// 最小座標点を取得
    pub fn min(&self) -> Point2D<T> {
        self.min
    }

    /// 最大座標点を取得
    pub fn max(&self) -> Point2D<T> {
        self.max
    }

    /// 座標成分での構築
    pub fn from_coords(min_x: T, min_y: T, max_x: T, max_y: T) -> Self {
        Self::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 中心点とサイズから構築
    pub fn from_center_size(center: Point2D<T>, width: T, height: T) -> Self {
        let half_width = width / (T::ONE + T::ONE);
        let half_height = height / (T::ONE + T::ONE);
        Self::new(
            Point2D::new(center.x() - half_width, center.y() - half_height),
            Point2D::new(center.x() + half_width, center.y() + half_height),
        )
    }

    /// 点のコレクションから境界ボックスを構築
    pub fn from_points(points: &[Point2D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min_x = points[0].x();
        let mut min_y = points[0].y();
        let mut max_x = points[0].x();
        let mut max_y = points[0].y();

        for point in points.iter().skip(1) {
            if point.x() < min_x {
                min_x = point.x();
            }
            if point.x() > max_x {
                max_x = point.x();
            }
            if point.y() < min_y {
                min_y = point.y();
            }
            if point.y() > max_y {
                max_y = point.y();
            }
        }

        Some(Self::from_coords(min_x, min_y, max_x, max_y))
    }

    /// 幅を取得
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 高さを取得
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 面積を取得
    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    /// ペリメーター（周囲長）を取得
    pub fn perimeter(&self) -> T {
        (self.width() + self.height()) * (T::ONE + T::ONE)
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        Point2D::new(
            (self.min.x() + self.max.x()) / (T::ONE + T::ONE),
            (self.min.y() + self.max.y()) / (T::ONE + T::ONE),
        )
    }

    /// サイズをベクトルとして取得
    pub fn size(&self) -> Vector2D<T> {
        Vector2D::new(self.width(), self.height())
    }

    /// 点が境界ボックス内にあるかを判定
    pub fn contains_point(&self, point: Point2D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    /// 点が境界ボックス内にあるかを判定（境界を除く）
    pub fn contains_point_strict(&self, point: Point2D<T>) -> bool {
        point.x() > self.min.x()
            && point.x() < self.max.x()
            && point.y() > self.min.y()
            && point.y() < self.max.y()
    }

    /// 他の境界ボックスと交差するかを判定
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
    }

    /// 他の境界ボックスを完全に含むかを判定
    pub fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.min.y() <= other.min.y()
            && self.max.x() >= other.max.x()
            && self.max.y() >= other.max.y()
    }

    /// 他の境界ボックスとの和集合
    pub fn union(&self, other: &Self) -> Self {
        Self::from_coords(
            if self.min.x() < other.min.x() { self.min.x() } else { other.min.x() },
            if self.min.y() < other.min.y() { self.min.y() } else { other.min.y() },
            if self.max.x() > other.max.x() { self.max.x() } else { other.max.x() },
            if self.max.y() > other.max.y() { self.max.y() } else { other.max.y() },
        )
    }

    /// 他の境界ボックスとの積集合
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min_x = if self.min.x() > other.min.x() { self.min.x() } else { other.min.x() };
        let min_y = if self.min.y() > other.min.y() { self.min.y() } else { other.min.y() };
        let max_x = if self.max.x() < other.max.x() { self.max.x() } else { other.max.x() };
        let max_y = if self.max.y() < other.max.y() { self.max.y() } else { other.max.y() };

        if min_x <= max_x && min_y <= max_y {
            Some(Self::from_coords(min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// 境界ボックスを拡張
    pub fn expand(&self, amount: T) -> Self {
        Self::from_coords(
            self.min.x() - amount,
            self.min.y() - amount,
            self.max.x() + amount,
            self.max.y() + amount,
        )
    }

    /// 境界ボックスをベクトル分だけ移動
    pub fn translate(&self, offset: Vector2D<T>) -> Self {
        Self::new(
            Point2D::new(self.min.x() + offset.x(), self.min.y() + offset.y()),
            Point2D::new(self.max.x() + offset.x(), self.max.y() + offset.y()),
        )
    }

    /// 境界ボックスをスケーリング
    pub fn scale(&self, factor: T) -> Self {
        let center = self.center();
        let half_width = self.width() * factor / (T::ONE + T::ONE);
        let half_height = self.height() * factor / (T::ONE + T::ONE);
        Self::from_center_size(center, half_width * (T::ONE + T::ONE), half_height * (T::ONE + T::ONE))
    }

    /// 4つの角の座標を取得
    pub fn corners(&self) -> [Point2D<T>; 4] {
        [
            self.min,
            Point2D::new(self.max.x(), self.min.y()),
            self.max,
            Point2D::new(self.min.x(), self.max.y()),
        ]
    }

    /// 空の境界ボックスかどうかを判定
    pub fn is_empty(&self) -> bool {
        self.width() == T::ZERO || self.height() == T::ZERO
    }

    /// 点から境界ボックスまでの最短距離
    pub fn distance_to_point(&self, point: Point2D<T>) -> T {
        if self.contains_point(point) {
            return T::ZERO;
        }

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

    /// f32からf64への変換
    pub fn to_f64(self) -> BoundingBox2D<f64>
    where
        T: Into<f64>,
    {
        BoundingBox2D::new(
            Point2D::new(self.min.x().into(), self.min.y().into()),
            Point2D::new(self.max.x().into(), self.max.y().into())
        )
    }

    /// f64からf32への変換
    pub fn to_f32(self) -> BoundingBox2D<f32>
    where
        T: Into<f32>,
    {
        BoundingBox2D::new(
            Point2D::new(self.min.x().into(), self.min.y().into()),
            Point2D::new(self.max.x().into(), self.max.y().into())
        )
    }
}

/// 3次元境界ボックス（軸平行境界ボックス - AABB）
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoundingBox3D<T: Scalar> {
    min: Point3D<T>,
    max: Point3D<T>,
}

impl<T: Scalar> BoundingBox3D<T> {
    /// 新しい3D境界ボックスを作成
    ///
    /// # Examples
    /// ```
    /// use geo_foundation::BoundingBox3D;
    /// use geo_foundation::Point3D;
    /// let bbox = BoundingBox3D::new(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(10.0, 5.0, 3.0)
    /// );
    /// ```
    pub fn new(min: Point3D<T>, max: Point3D<T>) -> Self {
        // min/maxの正規化
        let actual_min = Point3D::new(
            if min.x() <= max.x() { min.x() } else { max.x() },
            if min.y() <= max.y() { min.y() } else { max.y() },
            if min.z() <= max.z() { min.z() } else { max.z() },
        );
        let actual_max = Point3D::new(
            if min.x() >= max.x() { min.x() } else { max.x() },
            if min.y() >= max.y() { min.y() } else { max.y() },
            if min.z() >= max.z() { min.z() } else { max.z() },
        );
        Self {
            min: actual_min,
            max: actual_max,
        }
    }

    /// 最小座標点を取得
    pub fn min(&self) -> Point3D<T> {
        self.min
    }

    /// 最大座標点を取得
    pub fn max(&self) -> Point3D<T> {
        self.max
    }

    /// 座標成分での構築
    pub fn from_coords(min_x: T, min_y: T, min_z: T, max_x: T, max_y: T, max_z: T) -> Self {
        Self::new(Point3D::new(min_x, min_y, min_z), Point3D::new(max_x, max_y, max_z))
    }

    /// 中心点とサイズから構築
    pub fn from_center_size(center: Point3D<T>, width: T, height: T, depth: T) -> Self {
        let half_width = width / (T::ONE + T::ONE);
        let half_height = height / (T::ONE + T::ONE);
        let half_depth = depth / (T::ONE + T::ONE);
        Self::new(
            Point3D::new(
                center.x() - half_width,
                center.y() - half_height,
                center.z() - half_depth,
            ),
            Point3D::new(
                center.x() + half_width,
                center.y() + half_height,
                center.z() + half_depth,
            ),
        )
    }

    /// 点のコレクションから境界ボックスを構築
    pub fn from_points(points: &[Point3D<T>]) -> Option<Self> {
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
            if point.x() < min_x {
                min_x = point.x();
            }
            if point.x() > max_x {
                max_x = point.x();
            }
            if point.y() < min_y {
                min_y = point.y();
            }
            if point.y() > max_y {
                max_y = point.y();
            }
            if point.z() < min_z {
                min_z = point.z();
            }
            if point.z() > max_z {
                max_z = point.z();
            }
        }

        Some(Self::from_coords(min_x, min_y, min_z, max_x, max_y, max_z))
    }

    /// 幅（X軸方向）を取得
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 高さ（Y軸方向）を取得
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 奥行き（Z軸方向）を取得
    pub fn depth(&self) -> T {
        self.max.z() - self.min.z()
    }

    /// 体積を取得
    pub fn volume(&self) -> T {
        self.width() * self.height() * self.depth()
    }

    /// 表面積を取得
    pub fn surface_area(&self) -> T {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w * h + h * d + d * w) * (T::ONE + T::ONE)
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        Point3D::new(
            (self.min.x() + self.max.x()) / (T::ONE + T::ONE),
            (self.min.y() + self.max.y()) / (T::ONE + T::ONE),
            (self.min.z() + self.max.z()) / (T::ONE + T::ONE),
        )
    }

    /// サイズをベクトルとして取得
    pub fn size(&self) -> Vector3D<T> {
        Vector3D::new(self.width(), self.height(), self.depth())
    }

    /// 点が境界ボックス内にあるかを判定
    pub fn contains_point(&self, point: Point3D<T>) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
            && point.z() >= self.min.z()
            && point.z() <= self.max.z()
    }

    /// 点が境界ボックス内にあるかを判定（境界を除く）
    pub fn contains_point_strict(&self, point: Point3D<T>) -> bool {
        point.x() > self.min.x()
            && point.x() < self.max.x()
            && point.y() > self.min.y()
            && point.y() < self.max.y()
            && point.z() > self.min.z()
            && point.z() < self.max.z()
    }

    /// 他の境界ボックスと交差するかを判定
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x() <= other.max.x()
            && self.max.x() >= other.min.x()
            && self.min.y() <= other.max.y()
            && self.max.y() >= other.min.y()
            && self.min.z() <= other.max.z()
            && self.max.z() >= other.min.z()
    }

    /// 他の境界ボックスを完全に含むかを判定
    pub fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.min.y() <= other.min.y()
            && self.min.z() <= other.min.z()
            && self.max.x() >= other.max.x()
            && self.max.y() >= other.max.y()
            && self.max.z() >= other.max.z()
    }

    /// 他の境界ボックスとの和集合
    pub fn union(&self, other: &Self) -> Self {
        Self::from_coords(
            if self.min.x() < other.min.x() { self.min.x() } else { other.min.x() },
            if self.min.y() < other.min.y() { self.min.y() } else { other.min.y() },
            if self.min.z() < other.min.z() { self.min.z() } else { other.min.z() },
            if self.max.x() > other.max.x() { self.max.x() } else { other.max.x() },
            if self.max.y() > other.max.y() { self.max.y() } else { other.max.y() },
            if self.max.z() > other.max.z() { self.max.z() } else { other.max.z() },
        )
    }

    /// 他の境界ボックスとの積集合
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min_x = if self.min.x() > other.min.x() { self.min.x() } else { other.min.x() };
        let min_y = if self.min.y() > other.min.y() { self.min.y() } else { other.min.y() };
        let min_z = if self.min.z() > other.min.z() { self.min.z() } else { other.min.z() };
        let max_x = if self.max.x() < other.max.x() { self.max.x() } else { other.max.x() };
        let max_y = if self.max.y() < other.max.y() { self.max.y() } else { other.max.y() };
        let max_z = if self.max.z() < other.max.z() { self.max.z() } else { other.max.z() };

        if min_x <= max_x && min_y <= max_y && min_z <= max_z {
            Some(Self::from_coords(min_x, min_y, min_z, max_x, max_y, max_z))
        } else {
            None
        }
    }

    /// 境界ボックスを拡張
    pub fn expand(&self, amount: T) -> Self {
        Self::from_coords(
            self.min.x() - amount,
            self.min.y() - amount,
            self.min.z() - amount,
            self.max.x() + amount,
            self.max.y() + amount,
            self.max.z() + amount,
        )
    }

    /// 境界ボックスをベクトル分だけ移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        Self::new(
            Point3D::new(
                self.min.x() + offset.x(),
                self.min.y() + offset.y(),
                self.min.z() + offset.z(),
            ),
            Point3D::new(
                self.max.x() + offset.x(),
                self.max.y() + offset.y(),
                self.max.z() + offset.z(),
            ),
        )
    }

    /// 境界ボックスをスケーリング
    pub fn scale(&self, factor: T) -> Self {
        let center = self.center();
        let half_width = self.width() * factor / (T::ONE + T::ONE);
        let half_height = self.height() * factor / (T::ONE + T::ONE);
        let half_depth = self.depth() * factor / (T::ONE + T::ONE);
        Self::from_center_size(
            center,
            half_width * (T::ONE + T::ONE),
            half_height * (T::ONE + T::ONE),
            half_depth * (T::ONE + T::ONE),
        )
    }

    /// 8つの角の座標を取得
    pub fn corners(&self) -> [Point3D<T>; 8] {
        [
            self.min,
            Point3D::new(self.max.x(), self.min.y(), self.min.z()),
            Point3D::new(self.min.x(), self.max.y(), self.min.z()),
            Point3D::new(self.max.x(), self.max.y(), self.min.z()),
            Point3D::new(self.min.x(), self.min.y(), self.max.z()),
            Point3D::new(self.max.x(), self.min.y(), self.max.z()),
            Point3D::new(self.min.x(), self.max.y(), self.max.z()),
            self.max,
        ]
    }

    /// 空の境界ボックスかどうかを判定
    pub fn is_empty(&self) -> bool {
        self.width() == T::ZERO || self.height() == T::ZERO || self.depth() == T::ZERO
    }

    /// 点から境界ボックスまでの最短距離
    pub fn distance_to_point(&self, point: Point3D<T>) -> T {
        if self.contains_point(point) {
            return T::ZERO;
        }

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

        let dz = if point.z() < self.min.z() {
            self.min.z() - point.z()
        } else if point.z() > self.max.z() {
            point.z() - self.max.z()
        } else {
            T::ZERO
        };

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D境界ボックスに投影（Z軸を破棄）
    pub fn to_2d(&self) -> BoundingBox2D<T> {
        BoundingBox2D::new(self.min.to_2d(), self.max.to_2d())
    }

    /// f32からf64への変換
    pub fn to_f64(self) -> BoundingBox3D<f64>
    where
        T: Into<f64>,
    {
        BoundingBox3D::new(
            Point3D::new(self.min.x().into(), self.min.y().into(), self.min.z().into()),
            Point3D::new(self.max.x().into(), self.max.y().into(), self.max.z().into())
        )
    }

    /// f64からf32への変換
    pub fn to_f32(self) -> BoundingBox3D<f32>
    where
        T: Into<f32>,
    {
        BoundingBox3D::new(
            Point3D::new(self.min.x().into(), self.min.y().into(), self.min.z().into()),
            Point3D::new(self.max.x().into(), self.max.y().into(), self.max.z().into())
        )
    }
}

// Display実装
impl<T: Scalar + Display> Display for BoundingBox2D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox2D[({}, {}) -> ({}, {})]", 
               self.min.x(), self.min.y(), self.max.x(), self.max.y())
    }
}

impl<T: Scalar + Display> Display for BoundingBox3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BBox3D[({}, {}, {}) -> ({}, {}, {})]", 
               self.min.x(), self.min.y(), self.min.z(),
               self.max.x(), self.max.y(), self.max.z())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox2d_creation() {
        let bbox = BoundingBox2D::from_coords(1.0, 2.0, 5.0, 4.0);
        assert_eq!(bbox.min(), Point2D::new(1.0, 2.0));
        assert_eq!(bbox.max(), Point2D::new(5.0, 4.0));
        assert_eq!(bbox.width(), 4.0);
        assert_eq!(bbox.height(), 2.0);
    }

    #[test]
    fn test_bbox2d_normalization() {
        // min/maxが逆になっている場合の正規化
        let bbox = BoundingBox2D::new(Point2D::new(5.0, 4.0), Point2D::new(1.0, 2.0));
        assert_eq!(bbox.min(), Point2D::new(1.0, 2.0));
        assert_eq!(bbox.max(), Point2D::new(5.0, 4.0));
    }

    #[test]
    fn test_bbox2d_center_size() {
        let center = Point2D::new(3.0, 3.0);
        let bbox = BoundingBox2D::from_center_size(center, 4.0, 2.0);
        assert_eq!(bbox.center(), center);
        assert_eq!(bbox.width(), 4.0);
        assert_eq!(bbox.height(), 2.0);
    }

    #[test]
    fn test_bbox2d_from_points() {
        let points = vec![
            Point2D::new(1.0, 1.0),
            Point2D::new(5.0, 3.0),
            Point2D::new(2.0, 6.0),
            Point2D::new(4.0, 2.0),
        ];
        
        let bbox = BoundingBox2D::from_points(&points).unwrap();
        assert_eq!(bbox.min(), Point2D::new(1.0, 1.0));
        assert_eq!(bbox.max(), Point2D::new(5.0, 6.0));
    }

    #[test]
    fn test_bbox2d_contains_point() {
        let bbox = BoundingBox2D::from_coords(1.0, 1.0, 5.0, 4.0);
        
        assert!(bbox.contains_point(Point2D::new(3.0, 2.0))); // 内部
        assert!(bbox.contains_point(Point2D::new(1.0, 1.0))); // 角
        assert!(bbox.contains_point(Point2D::new(3.0, 1.0))); // 境界
        assert!(!bbox.contains_point(Point2D::new(0.0, 2.0))); // 外部
        
        assert!(!bbox.contains_point_strict(Point2D::new(1.0, 1.0))); // 境界除外
        assert!(bbox.contains_point_strict(Point2D::new(3.0, 2.0))); // 内部
    }

    #[test]
    fn test_bbox2d_intersects() {
        let bbox1 = BoundingBox2D::from_coords(1.0, 1.0, 5.0, 4.0);
        let bbox2 = BoundingBox2D::from_coords(3.0, 2.0, 7.0, 6.0);
        let bbox3 = BoundingBox2D::from_coords(6.0, 1.0, 8.0, 3.0);
        
        assert!(bbox1.intersects(&bbox2)); // 交差
        assert!(!bbox1.intersects(&bbox3)); // 非交差
    }

    #[test]
    fn test_bbox2d_union_intersection() {
        let bbox1 = BoundingBox2D::from_coords(1.0, 1.0, 4.0, 3.0);
        let bbox2 = BoundingBox2D::from_coords(2.0, 2.0, 5.0, 5.0);
        
        let union = bbox1.union(&bbox2);
        assert_eq!(union.min(), Point2D::new(1.0, 1.0));
        assert_eq!(union.max(), Point2D::new(5.0, 5.0));
        
        let intersection = bbox1.intersection(&bbox2).unwrap();
        assert_eq!(intersection.min(), Point2D::new(2.0, 2.0));
        assert_eq!(intersection.max(), Point2D::new(4.0, 3.0));
    }

    #[test]
    fn test_bbox3d_creation() {
        let bbox = BoundingBox3D::from_coords(1.0, 2.0, 3.0, 5.0, 4.0, 6.0);
        assert_eq!(bbox.min(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.max(), Point3D::new(5.0, 4.0, 6.0));
        assert_eq!(bbox.width(), 4.0);
        assert_eq!(bbox.height(), 2.0);
        assert_eq!(bbox.depth(), 3.0);
        assert_eq!(bbox.volume(), 24.0);
    }

    #[test]
    fn test_bbox3d_corners() {
        let bbox = BoundingBox3D::from_coords(0.0, 0.0, 0.0, 2.0, 2.0, 2.0);
        let corners = bbox.corners();
        assert_eq!(corners.len(), 8);
        assert_eq!(corners[0], Point3D::new(0.0, 0.0, 0.0)); // min
        assert_eq!(corners[7], Point3D::new(2.0, 2.0, 2.0)); // max
    }

    #[test]
    fn test_f32_f64_compatibility() {
        let bbox_f32 = BoundingBox2D::<f32>::from_coords(1.0, 2.0, 5.0, 4.0);
        let bbox_f64 = bbox_f32.to_f64();
        
        assert_eq!(bbox_f64.width(), 4.0f64);
        assert_eq!(bbox_f64.area(), 8.0f64);
    }

    #[test]
    fn test_empty_bbox() {
        let bbox = BoundingBox2D::from_coords(1.0, 1.0, 1.0, 1.0);
        assert!(bbox.is_empty());
        assert_eq!(bbox.area(), 0.0);
    }

    #[test]
    fn test_distance_to_point() {
        let bbox = BoundingBox2D::from_coords(1.0, 1.0, 3.0, 3.0);
        
        // 内部の点
        assert_eq!(bbox.distance_to_point(Point2D::new(2.0, 2.0)), 0.0);
        
        // 外部の点
        let distance = bbox.distance_to_point(Point2D::new(5.0, 2.0));
        assert!((distance - 2.0).abs() < 1e-10);
    }
}