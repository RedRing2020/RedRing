//! Aabb2Dトレイト定義
//!
//! 2次元軸平行境界ボックスの抽象インターフェース

use crate::Scalar;

pub trait Aabb2DTrait<T: Scalar> {
    type Point2D;

    /// 最小点を取得
    fn min(&self) -> Self::Point2D;
    /// 最大点を取得
    fn max(&self) -> Self::Point2D;
    /// 幅を取得
    fn width(&self) -> T;
    /// 高さを取得
    fn height(&self) -> T;
    /// 面積を取得
    fn area(&self) -> T;
    /// 中心点を取得
    fn center(&self) -> Self::Point2D;
    /// 点の包含判定
    fn contains_point(&self, point: &Self::Point2D) -> bool;
    /// 他のAABBの包含判定
    fn contains_bbox(&self, other: &Self) -> bool;
    /// 有効なAABBか判定
    fn is_valid(&self) -> bool;
    // 必要に応じて追加
}
