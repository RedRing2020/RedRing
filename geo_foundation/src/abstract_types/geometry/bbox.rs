//! BBox - 境界ボックスの最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本BBoxトレイト = 境界の基本属性のみ
//! ```text
//! BBox Trait = 基本属性のみ
//! ├── 範囲アクセス (min, max, center)
//! ├── 基本生成 (new)
//! ├── 体積計算 (volume)
//! └── 妥当性判定 (is_valid)
//!
//! 除外される責務:
//! ├── 包含判定 (contains_point, intersects) → BBoxContainment
//! ├── 集合演算 (union, intersection) → BBoxOperations
//! ├── 変換操作 (expand, transform) → BBoxTransform
//! └── 衝突判定 (fast_overlaps, separation) → CollisionBBox
//! ```
//!
//! ## 機能別拡張トレイトによる分離
//! ```text
//! BBoxContainment: 包含・交差判定
//! BBoxOperations: 集合演算（和・積・差）
//! BBoxTransform: 変換・拡張操作
//! CollisionBBox: 高速衝突判定
//! ```

/// 境界ボックスの最小責務トレイト
///
/// 境界の基本属性（最小・最大点、中心、体積）のみを提供。
/// 判定や演算などの機能は拡張トレイトで分離。
pub trait BBox<T: crate::Scalar> {
    /// 点の型（Point2D<T>, Point3D<T>など）
    type Point;

    /// 最小点を取得（左下奥など）
    fn min(&self) -> Self::Point;

    /// 最大点を取得（右上手前など）
    fn max(&self) -> Self::Point;

    /// 新しい境界ボックスを作成
    fn new(min: Self::Point, max: Self::Point) -> Self;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 体積/面積を計算
    fn volume(&self) -> T;

    /// 境界ボックスが有効かチェック（min <= max）
    fn is_valid(&self) -> bool;
}

/// 境界ボックスの基本操作トレイト
pub trait BBoxOps<T: crate::Scalar> {
    type Point;

    /// 点が境界ボックス内に含まれるかを判定
    fn contains_point(&self, point: Self::Point) -> bool;

    /// 他の境界ボックスと交差するかを判定
    fn intersects(&self, other: &Self) -> bool;

    /// 境界ボックスの中心点を取得
    fn center(&self) -> Self::Point;

    /// 境界ボックスの面積または体積を取得
    fn area_or_volume(&self) -> T;

    /// 境界ボックスを拡張
    fn expand(&self, amount: T) -> Self;
}

/// 境界ボックスの包含判定操作
pub trait BBoxContainment<T: crate::Scalar>: BBoxOps<T> {
    /// 点が境界ボックス内に含まれるかを判定（許容誤差付き）
    fn contains_point_with_tolerance(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 境界ボックスが他の境界ボックスを完全に含むかを判定
    fn contains_bbox(&self, other: &Self) -> bool;

    /// 境界ボックスが他の境界ボックスに完全に含まれるかを判定
    fn is_contained_by(&self, other: &Self) -> bool;
}

/// 境界ボックスの衝突判定操作
pub trait BBoxCollision<T: crate::Scalar>: BBoxContainment<T> {
    /// 高速な交差判定
    fn quick_intersect(&self, other: &Self) -> bool;

    /// 詳細な交差情報を取得
    fn intersection_area(&self, other: &Self) -> Option<T>;

    /// 境界ボックス間の距離を計算
    fn distance_to(&self, other: &Self) -> T;
}

/// 境界ボックスの変換操作
pub trait BBoxTransform<T: crate::Scalar> {
    type Vector;

    /// 境界ボックスを平行移動
    fn translate(&self, offset: &Self::Vector) -> Self;

    /// 境界ボックスをスケール
    fn scale(&self, factor: T) -> Self;

    /// 境界ボックスを拡張（ベクトル指定）
    fn expand_by_vector(&self, expansion: &Self::Vector) -> Self;
}

// 注意: 具体的な型エイリアスはgeo_primitivesで定義される
// 高次元境界データ（4次元以上）はAnalysisクレートで対応予定
// pub type BBox2D = geo_primitives::geometry2d::BBox2D;
// pub type BBox3D = geo_primitives::geometry3d::BBox3D;
