//! LineSegment Traits - 線分の最小責務抽象化
//!
//! abstracts 層での LineSegment 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義
//!
//! 設計: InfiniteLine → LineSegment の継承関係
//! LineSegment = InfiniteLine + trimming_info（2点間の線形補間）

use super::infinite_line_traits::{InfiniteLine2D, InfiniteLine3D};
use crate::Scalar;

/// 2D線分の最小責務抽象化
///
/// InfiniteLine + trimming_info の概念
/// 無限直線の一部をトリミングした線分
pub trait LineSegment2D<T: Scalar>: InfiniteLine2D<T> {
    /// 線分の開始点を取得
    fn start(&self) -> Self::Point;

    /// 線分の終了点を取得
    fn end(&self) -> Self::Point;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 無限直線上での開始パラメータ
    fn start_parameter(&self) -> T;

    /// 無限直線上での終了パラメータ
    fn end_parameter(&self) -> T;
}

/// 3D線分の最小責務抽象化
pub trait LineSegment3D<T: Scalar>: LineSegment2D<T> + InfiniteLine3D<T> {}

/// 線分の計量最小責務
pub trait LineSegmentMetrics<T: Scalar>: LineSegment2D<T> {
    /// 中点を取得
    fn midpoint(&self) -> Self::Point;

    /// パラメータ t での点を取得（0.0 = 開始点、1.0 = 終了点）
    fn point_at_parameter(&self, t: T) -> Self::Point;
}

/// 線分の包含判定最小責務
pub trait LineSegmentContainment<T: Scalar>: LineSegment2D<T> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}
