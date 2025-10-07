/// ジェネリックなバウンディングボックストレイト
///
/// 次元に依存しない境界ボックス操作の共通インターフェース

/// バウンディングボックスの基本操作を定義するトレイト
pub trait BoundingBox<const D: usize> {
    /// 座標型（通常はf64）
    type Coord;

    /// 最小座標を取得
    fn min(&self) -> [Self::Coord; D];

    /// 最大座標を取得
    fn max(&self) -> [Self::Coord; D];

    /// 新しいバウンディングボックスを作成
    fn new(min: [Self::Coord; D], max: [Self::Coord; D]) -> Self;

    /// 指定次元の幅を取得
    fn extent(&self, dim: usize) -> Self::Coord
    where
        Self::Coord: std::ops::Sub<Output = Self::Coord> + Copy;

    /// 体積/面積を計算
    fn volume(&self) -> Self::Coord
    where
        Self::Coord: std::ops::Mul<Output = Self::Coord> + Copy + From<f64>;

    /// 中心点を取得
    fn center(&self) -> [Self::Coord; D]
    where
        Self::Coord: std::ops::Add<Output = Self::Coord> + std::ops::Div<f64, Output = Self::Coord> + Copy;
}

/// バウンディングボックスの高度な操作を定義するトレイト
pub trait BoundingBoxOps<const D: usize>: BoundingBox<D> {
    /// 点が境界ボックス内にあるかチェック
    fn contains_point(&self, point: [Self::Coord; D]) -> bool
    where
        Self::Coord: PartialOrd + Copy;

    /// 他の境界ボックスと交差するかチェック
    fn intersects(&self, other: &Self) -> bool
    where
        Self::Coord: PartialOrd + Copy;

    /// 他の境界ボックスとの結合（和集合）
    fn union(&self, other: &Self) -> Self
    where
        Self::Coord: PartialOrd + Copy;

    /// 境界ボックスを指定量だけ拡張
    fn expand(&self, amount: Self::Coord) -> Self
    where
        Self::Coord: std::ops::Add<Output = Self::Coord> + std::ops::Sub<Output = Self::Coord> + Copy;

    /// 境界ボックスが有効かチェック（min <= max）
    fn is_valid(&self) -> bool
    where
        Self::Coord: PartialOrd + Copy;
}

/// 衝突判定用の特殊操作
pub trait CollisionBounds<const D: usize>: BoundingBoxOps<D> {
    /// 高速な重複テスト（軸平行境界ボックス特化）
    fn fast_overlaps(&self, other: &Self) -> bool
    where
        Self::Coord: PartialOrd + Copy;

    /// 境界ボックス間の距離（分離している場合のみ）
    fn separation_distance(&self, other: &Self) -> Option<Self::Coord>
    where
        Self::Coord: PartialOrd + std::ops::Sub<Output = Self::Coord> + Copy + From<f64>;

    /// 最も近い点を境界ボックス表面で取得
    fn closest_point_on_surface(&self, point: [Self::Coord; D]) -> [Self::Coord; D]
    where
        Self::Coord: PartialOrd + Copy;
}

/// 次元別のヘルパー型エイリアス
pub type BBox2D = crate::geometry2d::BBox2D;
pub type BBox3D = crate::geometry3d::BBox3D;


