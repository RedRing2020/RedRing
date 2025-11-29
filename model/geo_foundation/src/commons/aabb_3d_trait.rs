//! Aabb3Dトレイト定義
//!
//! 3次元軸平行境界ボックスの抽象インターフェース

use crate::Scalar;

pub trait Aabb3DTrait<T: Scalar> {
    type Point3D;

    /// 最小点を取得
    fn min(&self) -> Self::Point3D;
    /// 最大点を取得
    fn max(&self) -> Self::Point3D;
    /// 幅を取得
    fn width(&self) -> T;
    /// 高さを取得
    fn height(&self) -> T;
    /// 奥行きを取得
    fn depth(&self) -> T;
    /// 体積を取得
    fn volume(&self) -> T;
    /// 中心点を取得
    fn center(&self) -> Self::Point3D;
    /// 点の包含判定
    fn contains_point(&self, point: &Self::Point3D) -> bool;
    /// 他のAABBの包含判定
    fn contains_bbox(&self, other: &Self) -> bool;
    /// 他のAABBと交差するか
    fn intersects(&self, other: &Self) -> bool;
    /// 有効なAABBか判定
    fn is_valid(&self) -> bool;
    // 必要に応じて追加
}
