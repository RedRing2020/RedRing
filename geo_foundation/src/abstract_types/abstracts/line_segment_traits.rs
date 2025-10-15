//! LineSegment Traits - 線分の最小責務抽象化
//!
//! abstracts 層での LineSegment 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 2D線分の最小責務抽象化
///
/// 実装に依存しない純粋なインターフェース定義
pub trait LineSegment2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 線分の始点を取得
    fn start(&self) -> Self::Point;

    /// 線分の終点を取得
    fn end(&self) -> Self::Point;
}

/// 3D線分の最小責務抽象化
pub trait LineSegment3D<T: Scalar>: LineSegment2D<T> {
    /// ベクトル型
    type Vector;

    /// 線分の方向ベクトルを取得
    fn direction(&self) -> Self::Vector;
}

/// 線分の計量最小責務
pub trait LineSegmentMetrics<T: Scalar>: LineSegment2D<T> {
    /// 線分の長さを計算
    fn length(&self) -> T;

    /// 中点を取得
    fn midpoint(&self) -> Self::Point;
}

/// 線分の包含判定最小責務
pub trait LineSegmentContainment<T: Scalar>: LineSegment2D<T> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}