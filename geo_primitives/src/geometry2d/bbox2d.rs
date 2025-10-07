/// 2D Bounding Box - 2次元境界ボックス
///
/// 2D形状処理と衝突判定のための2次元境界ボックス

use crate::traits::bbox_trait::{BoundingBoxOps, CollisionBounds};
use crate::geometry2d::Point2D;

/// 2D軸平行境界ボックス（AABB: Axis-Aligned Bounding Box）
#[derive(Debug, Clone, PartialEq)]
pub struct BBox2D {
    pub min: Point2D,
    pub max: Point2D,
}

impl crate::traits::bbox_trait::BoundingBox<2> for BBox2D {
    type Coord = f64;

    fn min(&self) -> [Self::Coord; 2] {
        [self.min.x(), self.min.y()]
    }

    fn max(&self) -> [Self::Coord; 2] {
        [self.max.x(), self.max.y()]
    }

    fn new(min: [Self::Coord; 2], max: [Self::Coord; 2]) -> Self {
        Self {
            min: Point2D::new(min[0], min[1]),
            max: Point2D::new(max[0], max[1]),
        }
    }

    fn extent(&self, dim: usize) -> Self::Coord {
        match dim {
            0 => self.max.x() - self.min.x(),
            1 => self.max.y() - self.min.y(),
            _ => 0.0,
        }
    }

    fn volume(&self) -> Self::Coord {
        // 2Dでは面積
        self.width() * self.height()
    }

    fn center(&self) -> [Self::Coord; 2] {
        [
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
        ]
    }
}

impl BoundingBoxOps<2> for BBox2D {
    fn contains_point(&self, point: [Self::Coord; 2]) -> bool {
        point[0] >= self.min.x() && point[0] <= self.max.x() &&
        point[1] >= self.min.y() && point[1] <= self.max.y()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.max.x() >= other.min.x() && self.min.x() <= other.max.x() &&
        self.max.y() >= other.min.y() && self.min.y() <= other.max.y()
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            min: Point2D::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
            ),
            max: Point2D::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
            ),
        }
    }

    fn expand(&self, amount: Self::Coord) -> Self {
        Self {
            min: Point2D::new(self.min.x() - amount, self.min.y() - amount),
            max: Point2D::new(self.max.x() + amount, self.max.y() + amount),
        }
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y()
    }
}

impl CollisionBounds<2> for BBox2D {
    fn fast_overlaps(&self, other: &Self) -> bool {
        // 軸平行境界ボックス特化の高速重複テスト
        !(self.max.x() < other.min.x() || other.max.x() < self.min.x() ||
          self.max.y() < other.min.y() || other.max.y() < self.min.y())
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

        Some(max_separation)
    }

    fn closest_point_on_surface(&self, point: [Self::Coord; 2]) -> [Self::Coord; 2] {
        [
            point[0].clamp(self.min.x(), self.max.x()),
            point[1].clamp(self.min.y(), self.max.y()),
        ]
    }
}

impl BBox2D {
    /// 新しいBBox2Dを作成
    pub fn new(min: (f64, f64), max: (f64, f64)) -> Self {
        Self {
            min: Point2D::new(min.0, min.1),
            max: Point2D::new(max.0, max.1),
        }
    }

    /// Point2Dから2Dバウンディングボックスを作成
    pub fn from_points(min: Point2D, max: Point2D) -> Self {
        Self { min, max }
    }

    /// 点の集合からバウンディングボックスを作成
    pub fn from_point_array(points: &[Point2D]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let first = &points[0];
        let mut min = *first;
        let mut max = *first;

        for point in points.iter().skip(1) {
            min = Point2D::new(
                min.x().min(point.x()),
                min.y().min(point.y()),
            );
            max = Point2D::new(
                max.x().max(point.x()),
                max.y().max(point.y()),
            );
        }

        Some(Self::from_points(min, max))
    }

    /// 幅を取得
    pub fn width(&self) -> f64 {
        self.max.x() - self.min.x()
    }

    /// 高さを取得
    pub fn height(&self) -> f64 {
        self.max.y() - self.min.y()
    }

    /// 面積を取得（volumeのエイリアス）
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// 中心点をタプルで取得（互換性のため）
    pub fn center_tuple(&self) -> (f64, f64) {
        (
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
        )
    }

    /// 点が境界ボックス内にあるかチェック（タプル版）
    pub fn contains_point_tuple(&self, point: (f64, f64)) -> bool {
        self.contains_point([point.0, point.1])
    }

    /// 周囲長を計算
    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width() + self.height())
    }

    /// 対角線の長さを計算
    pub fn diagonal_length(&self) -> f64 {
        let w = self.width();
        let h = self.height();
        (w * w + h * h).sqrt()
    }

    /// 3Dバウンディングボックスに変換（Z=0）
    pub fn to_3d(&self) -> crate::geometry3d::BBox3D {
        crate::geometry3d::BBox3D::new(
            (self.min.x(), self.min.y(), 0.0),
            (self.max.x(), self.max.y(), 0.0),
        )
    }

    /// 正方形かどうかをチェック
    pub fn is_square(&self, tolerance: f64) -> bool {
        (self.width() - self.height()).abs() < tolerance
    }

    /// アスペクト比を計算（width / height）
    pub fn aspect_ratio(&self) -> f64 {
        let h = self.height();
        if h == 0.0 {
            f64::INFINITY
        } else {
            self.width() / h
        }
    }
}

// テストコードはunit_tests/bbox2d_tests.rsに移動
