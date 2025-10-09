//! 2D Bounding Box - 2次元境界ボックス
//!
//! 2D形状処理と衝突判定のための2次元境界ボックス

use crate::geometry2d::Point2D;
use geo_foundation::abstract_types::{
    geometry::{BBox as BBoxTrait, BBoxOps, CollisionBBox},
    Scalar,
};

/// 2D軸平行境界ボックス（AABB: Axis-Aligned Bounding Box）
///
/// 2次元空間での矩形による境界表現を提供します。
/// minとmaxのPointで矩形を定義し、CAD/CAMでの形状処理と衝突判定に使用されます。
///
/// # カプセル化
/// フィールドはprivateで、アクセサメソッドを通じてアクセスします。
#[derive(Debug, Clone, PartialEq)]
pub struct BBox2D<T: Scalar> {
    min: Point2D<T>,
    max: Point2D<T>,
}

impl<T: Scalar> BBoxTrait<T> for BBox2D<T> {
    type Point = Point2D<T>;

    fn min(&self) -> Self::Point {
        self.min
    }

    fn max(&self) -> Self::Point {
        self.max
    }

    fn new(min: Self::Point, max: Self::Point) -> Self {
        // 入力検証：min <= max であることを確認
        assert!(
            min.x() <= max.x() && min.y() <= max.y(),
            "Invalid bounding box: min point ({:?}) must be <= max point ({:?})",
            min,
            max
        );
        Self { min, max }
    }

    fn center(&self) -> Self::Point {
        let two = T::from_f64(2.0);
        Point2D::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    fn volume(&self) -> T {
        // 2Dでは面積
        self.width() * self.height()
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y()
    }
}

impl<T: Scalar> BBoxOps<T> for BBox2D<T>
where
    T: PartialOrd + Copy,
{
    fn contains_point(&self, point: Self::Point) -> bool {
        point.x() >= self.min.x()
            && point.x() <= self.max.x()
            && point.y() >= self.min.y()
            && point.y() <= self.max.y()
    }

    fn intersects(&self, other: &Self) -> bool {
        self.max.x() >= other.min.x()
            && self.min.x() <= other.max.x()
            && self.max.y() >= other.min.y()
            && self.min.y() <= other.max.y()
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

    fn expand(&self, amount: T) -> Self {
        Self {
            min: Point2D::new(self.min.x() - amount, self.min.y() - amount),
            max: Point2D::new(self.max.x() + amount, self.max.y() + amount),
        }
    }
}

impl<T: Scalar> CollisionBBox<T> for BBox2D<T> {
    fn fast_overlaps(&self, other: &Self) -> bool {
        // 軸平行境界ボックス特化の高速重複テスト
        !(self.max.x() < other.min.x()
            || other.max.x() < self.min.x()
            || self.max.y() < other.min.y()
            || other.max.y() < self.min.y())
    }

    fn separation_distance(&self, other: &Self) -> Option<T> {
        if self.intersects(other) {
            return None; // 重複している場合は分離距離なし
        }

        // 最近点間の距離を計算
        let self_closest = self.closest_point_on_surface(other.center());
        let other_closest = other.closest_point_on_surface(self.center());

        // 2点間のユークリッド距離
        let dx = self_closest.x() - other_closest.x();
        let dy = self_closest.y() - other_closest.y();
        let distance_squared = dx * dx + dy * dy;

        Some(distance_squared.sqrt())
    }

    fn closest_point_on_surface(&self, point: Self::Point) -> Self::Point {
        Point2D::new(
            point.x().clamp(self.min.x(), self.max.x()),
            point.y().clamp(self.min.y(), self.max.y()),
        )
    }
}

impl<T: Scalar> BBox2D<T> {
    /// 最小点を取得（読み取り専用アクセサ）
    pub fn min_point(&self) -> Point2D<T> {
        self.min
    }

    /// 最大点を取得（読み取り専用アクセサ）
    pub fn max_point(&self) -> Point2D<T> {
        self.max
    }

    /// 最小座標を取得（x座標）
    pub fn min_x(&self) -> T {
        self.min.x()
    }

    /// 最小座標を取得（y座標）
    pub fn min_y(&self) -> T {
        self.min.y()
    }

    /// 最大座標を取得（x座標）
    pub fn max_x(&self) -> T {
        self.max.x()
    }

    /// 最大座標を取得（y座標）
    pub fn max_y(&self) -> T {
        self.max.y()
    }

    /// 境界ボックスの更新（新しいminとmax点を設定）
    ///
    /// # 安全性
    /// min <= max の条件を満たさない場合はpanicします
    pub fn update(&mut self, min: Point2D<T>, max: Point2D<T>) {
        assert!(
            min.x() <= max.x() && min.y() <= max.y(),
            "Invalid bounding box: min must be <= max"
        );
        self.min = min;
        self.max = max;
    }

    /// 点を境界ボックスに含めるよう拡張
    pub fn expand_to_include_point(&mut self, point: Point2D<T>) {
        self.min = Point2D::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
        self.max = Point2D::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
    }

    /// 新しいBBoxをタプルから作成（互換性のため）
    pub fn new_from_tuples(min: (T, T), max: (T, T)) -> Self {
        Self::new(Point2D::new(min.0, min.1), Point2D::new(max.0, max.1))
    }

    /// 座標値から直接作成（互換性のため）
    pub fn from_coords(min_x: T, min_y: T, max_x: T, max_y: T) -> Self {
        Self::new(Point2D::new(min_x, min_y), Point2D::new(max_x, max_y))
    }

    /// 2つの点からBBoxを作成（タプル用の便利コンストラクタ）
    pub fn from_two_points(min: Point2D<T>, max: Point2D<T>) -> Self {
        Self::new(min, max)
    }

    /// 2つの点からBBoxを作成（順序を自動修正）
    pub fn from_two_points_safe(p1: Point2D<T>, p2: Point2D<T>) -> Self {
        let min = Point2D::new(p1.x().min(p2.x()), p1.y().min(p2.y()));
        let max = Point2D::new(p1.x().max(p2.x()), p1.y().max(p2.y()));
        Self::new(min, max)
    }

    /// 座標値から安全にBBoxを作成（順序を自動修正）
    pub fn from_coords_safe(x1: T, y1: T, x2: T, y2: T) -> Self {
        let min = Point2D::new(x1.min(x2), y1.min(y2));
        let max = Point2D::new(x1.max(x2), y1.max(y2));
        Self::new(min, max)
    }

    /// 点の集合からバウンディングボックスを作成
    pub fn from_point_array(points: &[Point2D<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let first = &points[0];
        let mut min = *first;
        let mut max = *first;

        for point in points.iter().skip(1) {
            min = Point2D::new(min.x().min(point.x()), min.y().min(point.y()));
            max = Point2D::new(max.x().max(point.x()), max.y().max(point.y()));
        }

        Some(Self::new(min, max))
    }

    /// 便利なfrom_pointsエイリアス
    pub fn from_points(points: &[Point2D<T>]) -> Option<Self> {
        Self::from_point_array(points)
    }

    /// 幅を取得
    pub fn width(&self) -> T {
        self.max.x() - self.min.x()
    }

    /// 高さを取得
    pub fn height(&self) -> T {
        self.max.y() - self.min.y()
    }

    /// 面積を取得（volumeのエイリアス）
    pub fn area(&self) -> T {
        self.width() * self.height()
    }

    /// 中心点をタプルで取得（互換性のため）
    pub fn center_tuple(&self) -> (T, T) {
        let center = self.center();
        (center.x(), center.y())
    }

    /// 点が境界ボックス内にあるかチェック（タプル版、互換性のため）
    pub fn contains_point_tuple(&self, point: (T, T)) -> bool {
        self.contains_point(Point2D::new(point.0, point.1))
    }

    /// 周囲長を計算
    pub fn perimeter(&self) -> T {
        let two = T::from_f64(2.0);
        two * (self.width() + self.height())
    }

    /// 対角線の長さを計算
    pub fn diagonal_length(&self) -> T
    where
        T: geo_foundation::abstract_types::Scalar,
    {
        let w = self.width();
        let h = self.height();
        let sum = w * w + h * h;
        // Scalarトレイトのsqrt機能を使用
        sum.sqrt()
    }

    /// 正方形かどうかをチェック
    pub fn is_square(&self, tolerance: T) -> bool {
        (self.width() - self.height()).abs() < tolerance
    }

    /// アスペクト比を計算（width / height）
    pub fn aspect_ratio(&self) -> Option<T>
    where
        T: PartialEq,
    {
        let h = self.height();
        if h == T::ZERO {
            None // 無限大の代わりにNoneを返す
        } else {
            Some(self.width() / h)
        }
    }
}

// f64版の特化実装（BBox2DExt相当）
impl BBox2D<f64> {
    /// 3Dバウンディングボックスに変換（Z=0）
    pub fn to_3d(&self) -> crate::geometry3d::BBox3D<f64> {
        use crate::geometry3d::Point3D;
        crate::geometry3d::BBox3D::new(
            Point3D::new(self.min.x(), self.min.y(), 0.0),
            Point3D::new(self.max.x(), self.max.y(), 0.0),
        )
    }

    /// 対角線の長さを計算（f64特化版）
    pub fn diagonal_length_f64(&self) -> f64 {
        let w = self.width();
        let h = self.height();
        (w * w + h * h).sqrt()
    }

    /// アスペクト比を計算（f64特化版）
    pub fn aspect_ratio_f64(&self) -> f64 {
        let h = self.height();
        if h == 0.0 {
            f64::INFINITY
        } else {
            self.width() / h
        }
    }
}

// f32版の特化実装（BBox2DExt相当）
impl BBox2D<f32> {
    /// 3Dバウンディングボックスに変換（Z=0、f64に変換）
    pub fn to_3d(&self) -> crate::geometry3d::BBox3D<f64> {
        use crate::geometry3d::Point3D;
        crate::geometry3d::BBox3D::new(
            Point3D::new(self.min.x() as f64, self.min.y() as f64, 0.0),
            Point3D::new(self.max.x() as f64, self.max.y() as f64, 0.0),
        )
    }

    /// 対角線の長さを計算（f32特化版）
    pub fn diagonal_length_f32(&self) -> f32 {
        let w = self.width();
        let h = self.height();
        (w * w + h * h).sqrt()
    }

    /// アスペクト比を計算（f32特化版）
    pub fn aspect_ratio_f32(&self) -> f32 {
        let h = self.height();
        if h == 0.0 {
            f32::INFINITY
        } else {
            self.width() / h
        }
    }
}

// 型エイリアス：命名統一と後方互換性
pub type BBox = BBox2D<f64>; // 旧BBox互換
pub type BBoxF64 = BBox2D<f64>;
pub type BBoxF32 = BBox2D<f32>;

// テストコードはunit_tests/BBox_tests.rsに移動
