//! BBox Traits - 境界ボックスの最小責務抽象化
//!
//! abstracts 層での BBox 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 2D境界ボックスの最小責務抽象化
///
/// 実装に依存しない純粋なインターフェース定義
pub trait BBox2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 最小点（左下）を取得
    fn min(&self) -> Self::Point;

    /// 最大点（右上）を取得
    fn max(&self) -> Self::Point;
}

/// 3D境界ボックスの最小責務抽象化
pub trait BBox3D<T: Scalar>: BBox2D<T> {
    /// 3D点の型
    type Point3D;

    /// 3D最小点を取得
    fn min_3d(&self) -> Self::Point3D;

    /// 3D最大点を取得
    fn max_3d(&self) -> Self::Point3D;
}

/// 境界ボックスの計量最小責務
pub trait BBoxMetrics<T: Scalar>: BBox2D<T> {
    /// 幅を計算
    fn width(&self) -> T;

    /// 高さを計算
    fn height(&self) -> T;

    /// 面積を計算
    fn area(&self) -> T;

    /// 中心点を取得
    fn center(&self) -> Self::Point;
}

/// 3D境界ボックスの計量最小責務
pub trait BBoxMetrics3D<T: Scalar>: BBox3D<T> + BBoxMetrics<T> {
    /// 奥行きを計算
    fn depth(&self) -> T;

    /// 体積を計算
    fn volume(&self) -> T;
}

/// 境界ボックスの包含判定最小責務
pub trait BBoxContainment<T: Scalar>: BBox2D<T> {
    /// 点が境界ボックス内にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 他の境界ボックスと重なるかを判定
    fn intersects(&self, other: &Self) -> bool;

    /// 他の境界ボックスを完全に含むかを判定
    fn contains_bbox(&self, other: &Self) -> bool;
}

/// 3D境界ボックスの包含判定最小責務
pub trait BBoxContainment3D<T: Scalar>: BBox3D<T> + BBoxContainment<T> {
    /// 3D点が境界ボックス内にあるかを判定
    fn contains_point_3d(&self, point: &Self::Point3D) -> bool;
}
