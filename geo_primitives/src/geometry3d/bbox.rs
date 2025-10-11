//! 3D Bounding Box - 衝突判定と形状処理のための3次元境界ボックス
//!
//! geometry3d配下に配置し、衝突判定のラフチェック対象として使用

use crate::geometry3d::Point;
use geo_foundation::{
    abstract_types::geometry::{
        BBox as BBoxTrait, BBoxCollision, BBoxContainment, BBoxOps, BBoxTransform,
    },
    Scalar,
};

/// 3D軸平行境界ボックス（AABB: Axis-Aligned Bounding Box）
///
/// 3次元空間での立方体による境界表現を提供します。
/// minとmaxのPointで立方体を定義し、CAD/CAMでの3D形状処理と衝突判定に使用されます。
///
/// # カプセル化
/// フィールドはprivateで、アクセサメソッドを通じてアクセスします。
#[derive(Debug, Clone, PartialEq)]
pub struct BBox3D<T: Scalar> {
    min: Point<T>,
    max: Point<T>,
}

impl<T: Scalar> BBoxTrait<T> for BBox3D<T> {
    type Point = Point<T>;

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
        let two = T::from_f64(2.0);
        Point::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
            (self.min.z() + self.max.z()) / two,
        )
    }

    fn volume(&self) -> T {
        // 3Dでは体積
        self.width() * self.height() * self.depth()
    }

    fn is_valid(&self) -> bool {
        self.min.x() <= self.max.x() && self.min.y() <= self.max.y() && self.min.z() <= self.max.z()
    }
}

impl<T: Scalar> BBoxOps<T> for BBox3D<T>
where
    T: PartialOrd + Copy,
{
    type Point = Point<T>;

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

    fn center(&self) -> Self::Point {
        BBoxTrait::center(self)
    }

    fn area_or_volume(&self) -> T {
        BBoxTrait::volume(self)
    }

    fn expand(&self, amount: T) -> Self {
        let half_amount = amount / (T::ONE + T::ONE);
        Self::new(
            Point::new(
                self.min.x() - half_amount,
                self.min.y() - half_amount,
                self.min.z() - half_amount,
            ),
            Point::new(
                self.max.x() + half_amount,
                self.max.y() + half_amount,
                self.max.z() + half_amount,
            ),
        )
    }
}

// AbstractBBoxトレイトの実装を追加
impl<T: Scalar> geo_foundation::abstract_types::geometry::primitive::AbstractBBox<T> for BBox3D<T> {
    type Point = Point<T>;

    fn min(&self) -> Self::Point {
        self.min
    }

    fn max(&self) -> Self::Point {
        self.max
    }
}

impl<T: Scalar> BBoxCollision<T> for BBox3D<T> {
    fn quick_intersect(&self, other: &Self) -> bool {
        // 軸平行境界ボックス特化の高速重複テスト
        !(self.max.x() < other.min.x()
            || other.max.x() < self.min.x()
            || self.max.y() < other.min.y()
            || other.max.y() < self.min.y()
            || self.max.z() < other.min.z()
            || other.max.z() < self.min.z())
    }

    fn intersection_area(&self, other: &Self) -> Option<T> {
        if !self.quick_intersect(other) {
            return None;
        }

        let min_x = self.min.x().max(other.min.x());
        let max_x = self.max.x().min(other.max.x());
        let min_y = self.min.y().max(other.min.y());
        let max_y = self.max.y().min(other.max.y());
        let min_z = self.min.z().max(other.min.z());
        let max_z = self.max.z().min(other.max.z());

        let width = max_x - min_x;
        let height = max_y - min_y;
        let depth = max_z - min_z;

        Some(width * height * depth)
    }

    fn distance_to(&self, other: &Self) -> T {
        if self.intersects(other) {
            return T::ZERO; // 重複している場合は距離は0
        }

        let dx = if self.max.x() < other.min.x() {
            other.min.x() - self.max.x()
        } else if other.max.x() < self.min.x() {
            self.min.x() - other.max.x()
        } else {
            T::ZERO
        };

        let dy = if self.max.y() < other.min.y() {
            other.min.y() - self.max.y()
        } else if other.max.y() < self.min.y() {
            self.min.y() - other.max.y()
        } else {
            T::ZERO
        };

        let dz = if self.max.z() < other.min.z() {
            other.min.z() - self.max.z()
        } else if other.max.z() < self.min.z() {
            self.min.z() - other.max.z()
        } else {
            T::ZERO
        };

        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

// BBoxContainment の実装
impl<T: Scalar> BBoxContainment<T> for BBox3D<T> {
    fn contains_point_with_tolerance(&self, point: &Self::Point, tolerance: T) -> bool {
        point.x() >= self.min.x() - tolerance
            && point.x() <= self.max.x() + tolerance
            && point.y() >= self.min.y() - tolerance
            && point.y() <= self.max.y() + tolerance
            && point.z() >= self.min.z() - tolerance
            && point.z() <= self.max.z() + tolerance
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        self.min.x() <= other.min.x()
            && self.min.y() <= other.min.y()
            && self.min.z() <= other.min.z()
            && self.max.x() >= other.max.x()
            && self.max.y() >= other.max.y()
            && self.max.z() >= other.max.z()
    }

    fn is_contained_by(&self, other: &Self) -> bool {
        other.contains_bbox(self)
    }
}

// BBoxTransform の実装
impl<T: Scalar> BBoxTransform<T> for BBox3D<T> {
    type Vector = crate::geometry3d::Vector<T>;

    fn translate(&self, offset: &Self::Vector) -> Self {
        Self::new(
            Point::new(
                self.min.x() + offset.x(),
                self.min.y() + offset.y(),
                self.min.z() + offset.z(),
            ),
            Point::new(
                self.max.x() + offset.x(),
                self.max.y() + offset.y(),
                self.max.z() + offset.z(),
            ),
        )
    }

    fn scale(&self, factor: T) -> Self {
        let center = <Self as geo_foundation::BBoxOps<T>>::center(self);
        let half_width = self.width() / (T::ONE + T::ONE) * factor;
        let half_height = self.height() / (T::ONE + T::ONE) * factor;
        let half_depth = self.depth() / (T::ONE + T::ONE) * factor;

        Self::new(
            Point::new(
                center.x() - half_width,
                center.y() - half_height,
                center.z() - half_depth,
            ),
            Point::new(
                center.x() + half_width,
                center.y() + half_height,
                center.z() + half_depth,
            ),
        )
    }

    fn expand_by_vector(&self, expansion: &Self::Vector) -> Self {
        Self::new(
            Point::new(
                self.min.x() - expansion.x(),
                self.min.y() - expansion.y(),
                self.min.z() - expansion.z(),
            ),
            Point::new(
                self.max.x() + expansion.x(),
                self.max.y() + expansion.y(),
                self.max.z() + expansion.z(),
            ),
        )
    }
}

impl<T: Scalar> BBox3D<T> {
    /// 最小点を取得（読み取り専用アクセサ）
    pub fn min_point(&self) -> Point<T> {
        self.min
    }

    /// 最大点を取得（読み取り専用アクセサ）
    pub fn max_point(&self) -> Point<T> {
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

    /// 最小座標を取得（z座標）
    pub fn min_z(&self) -> T {
        self.min.z()
    }

    /// 最大座標を取得（x座標）
    pub fn max_x(&self) -> T {
        self.max.x()
    }

    /// 最大座標を取得（y座標）
    pub fn max_y(&self) -> T {
        self.max.y()
    }

    /// 最大座標を取得（z座標）
    pub fn max_z(&self) -> T {
        self.max.z()
    }

    /// 境界ボックスの更新（新しいminとmax点を設定）
    ///
    /// # 安全性
    /// min <= max の条件を満たさない場合はpanicします
    pub fn update(&mut self, min: Point<T>, max: Point<T>) {
        assert!(
            min.x() <= max.x() && min.y() <= max.y() && min.z() <= max.z(),
            "Invalid bounding box: min must be <= max"
        );
        self.min = min;
        self.max = max;
    }

    /// 点を境界ボックスに含めるよう拡張
    pub fn expand_to_include_point(&mut self, point: Point<T>) {
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

    /// 新しいBBox3Dをタプルから作成（ジェネリック版）
    pub fn new_from_tuples(min: (T, T, T), max: (T, T, T)) -> Self {
        Self::new(
            Point::new(min.0, min.1, min.2),
            Point::new(max.0, max.1, max.2),
        )
    }

    /// 座標値から直接作成（ジェネリック版）
    pub fn from_coords(min_x: T, min_y: T, min_z: T, max_x: T, max_y: T, max_z: T) -> Self {
        Self::new(
            Point::new(min_x, min_y, min_z),
            Point::new(max_x, max_y, max_z),
        )
    }

    /// 2つの点からBBoxを作成（便利コンストラクタ）
    pub fn from_two_points(min: Point<T>, max: Point<T>) -> Self {
        Self::new(min, max)
    }

    /// 以前の名前との互換性のため
    pub fn from_3d_points(min: Point<T>, max: Point<T>) -> Self {
        Self::from_two_points(min, max)
    }

    /// 2D点から3Dバウンディングボックスを作成（Z=0、T::ZEROを使用）
    pub fn from_2d_points(
        min: crate::geometry2d::Point2DF64,
        max: crate::geometry2d::Point2DF64,
    ) -> Self
    where
        T: From<f64>,
    {
        Self::new(
            Point::new(T::from(min.x()), T::from(min.y()), T::ZERO),
            Point::new(T::from(max.x()), T::from(max.y()), T::ZERO),
        )
    }

    /// 点の集合からバウンディングボックスを作成
    pub fn from_point_array(points: &[Point<T>]) -> Option<Self> {
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
    pub fn from_points(points: &[Point<T>]) -> Option<Self> {
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

    /// 奥行きを取得
    pub fn depth(&self) -> T {
        self.max.z() - self.min.z()
    }

    /// 中心点をタプルで取得（互換性のため）
    pub fn center_tuple(&self) -> (T, T, T) {
        let center = <Self as geo_foundation::BBoxOps<T>>::center(self);
        (center.x(), center.y(), center.z())
    }

    /// 点が境界ボックス内にあるかチェック（タプル版、Tジェネリック版）
    pub fn contains_point_tuple(&self, point: (T, T, T)) -> bool {
        self.contains_point(Point::new(point.0, point.1, point.2))
    }

    /// 表面積を計算（ジェネリック版でT型を返す）
    pub fn surface_area(&self) -> T {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (T::ONE + T::ONE) * (w * h + w * d + h * d)
    }

    /// 対角線の長さを計算（ジェネリック版でT型を返す）
    pub fn diagonal_length(&self) -> T {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w * w + h * h + d * d).sqrt()
    }

    /// 立方体かどうかを判定（ジェネリック版でT型のtoleranceを使用）
    pub fn is_cube(&self, tolerance: T) -> bool {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w - h).abs() < tolerance && (h - d).abs() < tolerance
    }
}

// 旧名前との互換性のためのtype alias
#[deprecated(note = "Use BBox3D instead")]
pub type LegacyBoundingBox = BBox3D<f64>;

/// f64専用のBBox3Dエイリアス（Circle成功パターンに従って）
pub type BBox3DF64 = BBox3D<f64>;

/// f32専用のBBox3Dエイリアス
pub type BBox3DF32 = BBox3D<f32>;
