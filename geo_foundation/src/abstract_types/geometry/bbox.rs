//! 境界ボックス（Bounding Box）トレイト
//!
//! 2D/3D形状の立方体/矩形による境界表現の共通インターフェース

/// 境界ボックスの基本操作を定義するトレイト
///
/// 立方体/矩形による形状の境界表現を提供します。
/// 高次元データ（4次元以上）はAnalysisクレートで別途対応予定。
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

/// 境界ボックスの高度な操作を定義するトレイト
pub trait BBoxOps<T: crate::Scalar>: BBox<T> {
    /// 点が境界ボックス内にあるかチェック
    fn contains_point(&self, point: Self::Point) -> bool;

    /// 他の境界ボックスと交差するかチェック
    fn intersects(&self, other: &Self) -> bool;

    /// 他の境界ボックスとの結合（和集合）
    fn union(&self, other: &Self) -> Self;

    /// 境界ボックスを指定量だけ拡張
    fn expand(&self, amount: T) -> Self;
}

/// 衝突判定用の特殊操作
pub trait CollisionBBox<T: crate::Scalar>: BBoxOps<T> {
    /// 高速な重複テスト（軸平行境界ボックス特化）
    fn fast_overlaps(&self, other: &Self) -> bool;

    /// 境界ボックス間の分離距離（重複していない場合のみ）
    fn separation_distance(&self, other: &Self) -> Option<T>;

    /// 指定点に最も近い境界ボックス表面上の点を取得
    fn closest_point_on_surface(&self, point: Self::Point) -> Self::Point;
}

// 注意: 具体的な型エイリアスはgeo_primitivesで定義される
// 高次元境界データ（4次元以上）はAnalysisクレートで対応予定
// pub type BBox2D = geo_primitives::geometry2d::BBox2D;
// pub type BBox3D = geo_primitives::geometry3d::BBox3D;
