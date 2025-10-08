//! 3D Bounding Box - 衝突判定と形状処理のための3次元境界ボックス
//!
//! geometry3d配下に配置し、衝突判定のラフチェック対象として使用

use crate::geometry3d::Point;
use geo_foundation::abstract_types::geometry::{BBox as BBoxTrait, BBoxOps, CollisionBBox};

/// 3D軸平行境界ボックス（AABB: Axis-Aligned Bounding Box）
///
/// 3次元空間での立方体による境界表現を提供します。
/// minとmaxのPointで立方体を定義し、CAD/CAMでの3D形状処理と衝突判定に使用されます。
///
/// # カプセル化
/// フィールドはprivateで、アクセサメソッドを通じてアクセスします。
#[derive(Debug, Clone, PartialEq)]
pub struct BBox3D {
    min: Point,
    max: Point,
}

impl BBoxTrait<f64> for BBox3D {
    type Point = Point;

    fn min(&self) -> Self::Point {
        self.min
    }

    fn max(&self) -> Self::Point {
        self.max
    }

    fn new(min: Self::Point, max: Self::Point) -> Self {
        Self { min, max }
    }

    fn center(&self) -> Self::Point {
        Point::new(
            (self.min.x() + self.max.x()) / 2.0,
            (self.min.y() + self.max.y()) / 2.0,
            (self.min.z() + self.max.z()) / 2.0,
        )
    }

    fn volume(&self) -> f64 {
        // 3Dでは体積
        self.width() * self.height() * self.depth()
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y() && self.min.z() <= self.max.z()
    }
}

impl BBoxOps<f64> for BBox3D {
    fn contains_point(&self, point: Self::Point) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
            && point.z() >= self.min.z()
            && point.z() <= self.max.z()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.max.x() >= other.min.x()
            && self.min.x() <= other.max.x()
            && self.max.y() >= other.min.y()
            && self.min.y() <= other.max.y()
            && self.max.z() >= other.min.z()
            && self.min.z() <= other.max.z()
    }

    fn union(&self, other: &Self) -> Self {
        Self {
            min: Point::new(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
                self.min.z().min(other.min.z()),
            ),
            max: Point::new(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
                self.max.z().max(other.max.z()),
            ),
        }
    }

    fn expand(&self, amount: f64) -> Self {
        Self {
            min: Point::new(
                self.min.x() - amount,
                self.min.y() - amount,
                self.min.z() - amount,
            ),
            max: Point::new(
                self.max.x() + amount,
                self.max.y() + amount,
                self.max.z() + amount,
            ),
        }
    }
}

impl CollisionBBox<f64> for BBox3D {
    fn fast_overlaps(&self, other: &Self) -> bool {
        // 軸平行境界ボックス特化の高速重複テスト
        !(self.max.x() < other.min.x()
            || other.max.x() < self.min.x()
            || self.max.y() < other.min.y()
            || other.max.y() < self.min.y()
            || self.max.z() < other.min.z()
            || other.max.z() < self.min.z())
    }

    fn separation_distance(&self, other: &Self) -> Option<f64> {
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

    fn closest_point_on_surface(&self, point: Self::Point) -> Self::Point {
        Point::new(
            point.x().clamp(self.min.x(), self.max.x()),
            point.y().clamp(self.min.y(), self.max.y()),
            point.z().clamp(self.min.z(), self.max.z()),
        )
    }
}

impl BBox3D {
    /// 最小点を取得（読み取り専用アクセサ）
    pub fn min_point(&self) -> Point {
        self.min
    }

    /// 最大点を取得（読み取り専用アクセサ）
    pub fn max_point(&self) -> Point {
        self.max
    }

    /// 最小座標を取得（x座標）
    pub fn min_x(&self) -> f64 {
        self.min.x()
    }

    /// 最小座標を取得（y座標）
    pub fn min_y(&self) -> f64 {
        self.min.y()
    }

    /// 最小座標を取得（z座標）
    pub fn min_z(&self) -> f64 {
        self.min.z()
    }

    /// 最大座標を取得（x座標）
    pub fn max_x(&self) -> f64 {
        self.max.x()
    }

    /// 最大座標を取得（y座標）
    pub fn max_y(&self) -> f64 {
        self.max.y()
    }

    /// 最大座標を取得（z座標）
    pub fn max_z(&self) -> f64 {
        self.max.z()
    }

    /// 境界ボックスの更新（新しいminとmax点を設定）
    ///
    /// # 安全性
    /// min <= max の条件を満たさない場合はpanicします
    pub fn update(&mut self, min: Point, max: Point) {
        assert!(
            min.x() <= max.x() && min.y() <= max.y() && min.z() <= max.z(),
            "Invalid bounding box: min must be <= max"
        );
        self.min = min;
        self.max = max;
    }

    /// 点を境界ボックスに含めるよう拡張
    pub fn expand_to_include_point(&mut self, point: Point) {
        self.min = Point::new(
            self.min.x().min(point.x()),
            self.min.y().min(point.y()),
            self.min.z().min(point.z()),
        );
        self.max = Point::new(
            self.max.x().max(point.x()),
            self.max.y().max(point.y()),
            self.max.z().max(point.z()),
        );
    }

    /// 新しいBBox3Dをタプルから作成（互換性のため）
    pub fn new_from_tuples(min: (f64, f64, f64), max: (f64, f64, f64)) -> Self {
        Self::new(
            Point::new(min.0, min.1, min.2),
            Point::new(max.0, max.1, max.2),
        )
    }

    /// 座標値から直接作成（互換性のため）
    pub fn from_coords(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Self {
        Self::new(
            Point::new(min_x, min_y, min_z),
            Point::new(max_x, max_y, max_z),
        )
    }

    /// 2つの点からBBoxを作成（便利コンストラクタ）
    pub fn from_two_points(min: Point, max: Point) -> Self {
        Self::new(min, max)
    }

    /// 以前の名前との互換性のため
    pub fn from_3d_points(min: Point, max: Point) -> Self {
        Self::from_two_points(min, max)
    }

    /// 2D点から3Dバウンディングボックスを作成（Z=0）
    pub fn from_2d_points(
        min: crate::geometry2d::Point2DF64,
        max: crate::geometry2d::Point2DF64,
    ) -> Self {
        Self::new(
            Point::new(min.x(), min.y(), 0.0),
            Point::new(max.x(), max.y(), 0.0),
        )
    }

    /// 点の集合からバウンディングボックスを作成
    pub fn from_point_array(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let first = &points[0];
        let mut min = *first;
        let mut max = *first;

        for point in points.iter().skip(1) {
            min = Point::new(
                min.x().min(point.x()),
                min.y().min(point.y()),
                min.z().min(point.z()),
            );
            max = Point::new(
                max.x().max(point.x()),
                max.y().max(point.y()),
                max.z().max(point.z()),
            );
        }

        Some(Self::new(min, max))
    }

    /// 便利なfrom_pointsエイリアス
    pub fn from_points(points: &[Point]) -> Option<Self> {
        Self::from_point_array(points)
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
        let center = self.center();
        (center.x(), center.y(), center.z())
    }

    /// 点が境界ボックス内にあるかチェック（タプル版、互換性のため）
    pub fn contains_point_tuple(&self, point: (f64, f64, f64)) -> bool {
        self.contains_point(Point::new(point.0, point.1, point.2))
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

    /// 立方体かどうかを判定
    pub fn is_cube(&self, tolerance: f64) -> bool {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w - h).abs() < tolerance && (h - d).abs() < tolerance
    }
}

// 旧名前との互換性のためのtype alias
#[deprecated(note = "Use BBox3D instead")]
pub type LegacyBoundingBox = BBox3D;

// 型エイリアス：3D専用の明確化
pub type BBox3DF64 = BBox3D; // f64専用BBox3D（明示的）
