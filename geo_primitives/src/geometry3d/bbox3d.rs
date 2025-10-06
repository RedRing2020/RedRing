/// 3D Bounding Box - 衝突判定と形状処理のための3次元境界ボックス
/// 
/// geometry3d配下に配置し、衝突判定のラフチェック対象として使用

use crate::traits::bbox_trait::{BoundingBoxOps, CollisionBounds};
use crate::geometry2d::Point2D;
use crate::geometry3d::Point3D;

/// 3D軸平行境界ボックス（AABB: Axis-Aligned Bounding Box）
#[derive(Debug, Clone, PartialEq)]
pub struct BBox3D {
    pub min: Point3D,
    pub max: Point3D,
}

impl crate::traits::bbox_trait::BoundingBox<3> for BBox3D {
    type Coord = f64;

    fn min(&self) -> [Self::Coord; 3] {
        [self.min.x(), self.min.y(), self.min.z()]
    }

    fn max(&self) -> [Self::Coord; 3] {
        [self.max.x(), self.max.y(), self.max.z()]
    }

    fn new(min: [Self::Coord; 3], max: [Self::Coord; 3]) -> Self {
        Self {
            min: Point3D::new(min[0], min[1], min[2]),
            max: Point3D::new(max[0], max[1], max[2]),
        }
    }

    fn extent(&self, dim: usize) -> Self::Coord {
        match dim {
            0 => self.max.x() - self.min.x(),
            1 => self.max.y() - self.min.y(),
            2 => self.max.z() - self.min.z(),
            _ => 0.0,
        }
    }

    fn volume(&self) -> Self::Coord {
        self.width() * self.height() * self.depth()
    }

    fn center(&self) -> [Self::Coord; 3] {
        [
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
            (self.min.z() + self.max.z()) / 2.0,
        ]
    }
}

impl BoundingBoxOps<3> for BBox3D {
    fn contains_point(&self, point: [Self::Coord; 3]) -> bool {
        point[0] >= self.min.x() && point[0] <= self.max.x() &&
        point[1] >= self.min.y() && point[1] <= self.max.y() &&
        point[2] >= self.min.z() && point[2] <= self.max.z()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.max.x() >= other.min.x() && self.min.x() <= other.max.x() &&
        self.max.y() >= other.min.y() && self.min.y() <= other.max.y() &&
        self.max.z() >= other.min.z() && self.min.z() <= other.max.z()
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            min: Point3D::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
                self.min.z().min(other.min.z()),
            ),
            max: Point3D::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
                self.max.z().max(other.max.z()),
            ),
        }
    }

    fn expand(&self, amount: Self::Coord) -> Self {
        Self {
            min: Point3D::new(self.min.x() - amount, self.min.y() - amount, self.min.z() - amount),
            max: Point3D::new(self.max.x() + amount, self.max.y() + amount, self.max.z() + amount),
        }
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y() && self.min.z() <= self.max.z()
    }
}

impl CollisionBounds<3> for BBox3D {
    fn fast_overlaps(&self, other: &Self) -> bool {
        // 軸平行境界ボックス特化の高速重複テスト
        !(self.max.x() < other.min.x() || other.max.x() < self.min.x() ||
          self.max.y() < other.min.y() || other.max.y() < self.min.y() ||
          self.max.z() < other.min.z() || other.max.z() < self.min.z())
    }

    fn separation_distance(&self, other: &Self) -> Option<Self::Coord> {
        if self.intersects(other) {
            return None; // 重複している場合は分離距離なし
        }

        let mut max_separation = 0.0f64;
        
        // X軸での分離距離
        if self.max.x() < other.min.x() {
            max_separation = max_separation.max(other.min.x() - self.max.x());
        } else if other.max.x() < self.min.x() {
            max_separation = max_separation.max(self.min.x() - other.max.x());
        }
        
        // Y軸での分離距離
        if self.max.y() < other.min.y() {
            max_separation = max_separation.max(other.min.y() - self.max.y());
        } else if other.max.y() < self.min.y() {
            max_separation = max_separation.max(self.min.y() - other.max.y());
        }
        
        // Z軸での分離距離
        if self.max.z() < other.min.z() {
            max_separation = max_separation.max(other.min.z() - self.max.z());
        } else if other.max.z() < self.min.z() {
            max_separation = max_separation.max(self.min.z() - other.max.z());
        }

        Some(max_separation)
    }

    fn closest_point_on_surface(&self, point: [Self::Coord; 3]) -> [Self::Coord; 3] {
        [
            point[0].clamp(self.min.x(), self.max.x()),
            point[1].clamp(self.min.y(), self.max.y()),
            point[2].clamp(self.min.z(), self.max.z()),
        ]
    }
}

impl BBox3D {
    /// 新しいBBox3Dを作成
    pub fn new(min: (f64, f64, f64), max: (f64, f64, f64)) -> Self {
        Self {
            min: Point3D::new(min.0, min.1, min.2),
            max: Point3D::new(max.0, max.1, max.2),
        }
    }

    /// 2D点から3Dバウンディングボックスを作成（Z=0）
    pub fn from_2d_points(min: Point2D, max: Point2D) -> Self {
        Self {
            min: Point3D::new(min.x(), min.y(), 0.0),
            max: Point3D::new(max.x(), max.y(), 0.0),
        }
    }

    /// Point3Dから3Dバウンディングボックスを作成
    pub fn from_3d_points(min: Point3D, max: Point3D) -> Self {
        Self { min, max }
    }

    /// 点の集合からバウンディングボックスを作成
    pub fn from_points(points: &[Point3D]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let first = &points[0];
        let mut min = *first;
        let mut max = *first;

        for point in points.iter().skip(1) {
            min = Point3D::new(
                min.x().min(point.x()),
                min.y().min(point.y()),
                min.z().min(point.z()),
            );
            max = Point3D::new(
                max.x().max(point.x()),
                max.y().max(point.y()),
                max.z().max(point.z()),
            );
        }

        Some(Self::from_3d_points(min, max))
    }

    /// 幅を取得
    pub fn width(&self) -> f64 {
        self.max.x() - self.min.x()
    }

    /// 高さを取得
    pub fn height(&self) -> f64 {
        self.max.y() - self.min.y()
    }

    /// 奥行きを取得
    pub fn depth(&self) -> f64 {
        self.max.z() - self.min.z()
    }

    /// 中心点をタプルで取得（互換性のため）
    pub fn center_tuple(&self) -> (f64, f64, f64) {
        (
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
            (self.min.z() + self.max.z()) / 2.0,
        )
    }

    /// 点が境界ボックス内にあるかチェック（タプル版）
    pub fn contains_point_tuple(&self, point: (f64, f64, f64)) -> bool {
        self.contains_point([point.0, point.1, point.2])
    }

    /// 表面積を計算
    pub fn surface_area(&self) -> f64 {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        2.0 * (w * h + w * d + h * d)
    }

    /// 対角線の長さを計算
    pub fn diagonal_length(&self) -> f64 {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w * w + h * h + d * d).sqrt()
    }
}

// 旧BoundingBoxとの互換性のためのtype alias
#[deprecated(note = "Use BBox3D instead")]
pub type LegacyBoundingBox = BBox3D;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::bbox_trait::{BoundingBox, BoundingBoxOps, CollisionBounds};

    #[test]
    fn test_bbox3d_creation() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        assert_eq!(bbox.min, Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(bbox.max, Point3D::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_bbox3d_dimensions() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        assert_eq!(bbox.width(), 2.0);
        assert_eq!(bbox.height(), 3.0);
        assert_eq!(bbox.depth(), 4.0);
        assert_eq!(bbox.volume(), 24.0);
        assert_eq!(bbox.surface_area(), 52.0); // 2*(2*3 + 2*4 + 3*4) = 2*26 = 52
    }

    #[test]
    fn test_bbox3d_from_points() {
        let points = vec![
            Point3D::new(1.0, 2.0, 3.0),
            Point3D::new(-1.0, 5.0, 1.0),
            Point3D::new(3.0, 0.0, 4.0),
        ];
        
        let bbox = BBox3D::from_points(&points).unwrap();
        assert_eq!(bbox.min, Point3D::new(-1.0, 0.0, 1.0));
        assert_eq!(bbox.max, Point3D::new(3.0, 5.0, 4.0));
    }

    #[test]
    fn test_collision_bounds_interface() {
        let bbox1 = BBox3D::new((0.0, 0.0, 0.0), (2.0, 2.0, 2.0));
        let bbox2 = BBox3D::new((1.0, 1.0, 1.0), (3.0, 3.0, 3.0));
        let bbox3 = BBox3D::new((3.0, 3.0, 3.0), (4.0, 4.0, 4.0));
        
        // 高速重複テスト
        assert!(bbox1.fast_overlaps(&bbox2));
        assert!(!bbox1.fast_overlaps(&bbox3));
        
        // 分離距離
        assert!(bbox1.separation_distance(&bbox2).is_none()); // 重複している
        let sep_dist = bbox1.separation_distance(&bbox3).unwrap();
        assert_eq!(sep_dist, 1.0); // (3.0 - 2.0) = 1.0
        
        // 最近点
        let closest = bbox1.closest_point_on_surface([5.0, 1.0, 1.0]);
        assert_eq!(closest, [2.0, 1.0, 1.0]); // X軸でクランプ
    }

    #[test]
    fn test_generic_trait_implementation() {
        let bbox = BBox3D::new((0.0, 0.0, 0.0), (2.0, 3.0, 4.0));
        
        // BoundingBoxトレイト
        assert_eq!(bbox.min(), [0.0, 0.0, 0.0]);
        assert_eq!(bbox.max(), [2.0, 3.0, 4.0]);
        assert_eq!(bbox.extent(0), 2.0);
        assert_eq!(bbox.extent(1), 3.0);
        assert_eq!(bbox.extent(2), 4.0);
        assert_eq!(bbox.center(), [1.0, 1.5, 2.0]);
        
        // BoundingBoxOpsトレイト
        assert!(bbox.contains_point([1.0, 1.5, 2.0]));
        assert!(!bbox.contains_point([3.0, 1.0, 1.0]));
        assert!(bbox.is_valid());
        
        let expanded = bbox.expand(0.5);
        assert_eq!(expanded.min, Point3D::new(-0.5, -0.5, -0.5));
        assert_eq!(expanded.max, Point3D::new(2.5, 3.5, 4.5));
    }
}