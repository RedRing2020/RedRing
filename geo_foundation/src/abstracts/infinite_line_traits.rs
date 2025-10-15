//! InfiniteLine Traits - 無限直線の最小責務抽象化
//!
//! abstracts 層での InfiniteLine 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 無限直線の最小責務抽象化
///
/// LineSegmentの基盤となる無限直線概念
pub trait InfiniteLine2D<T: Scalar> {
    /// 点の型
    type Point;
    /// 方向ベクトルの型
    type Direction;

    /// 直線上の原点を取得
    fn origin(&self) -> Self::Point;

    /// 直線の方向ベクトルを取得
    fn direction(&self) -> Self::Direction;
}

/// 無限直線の最小責務抽象化（3D）
///
/// InfiniteLine2Dを3Dに拡張
pub trait InfiniteLine3D<T: Scalar>: InfiniteLine2D<T> {
    /// Z成分を含む方向ベクトル型
    type Direction3D;

    /// 3D方向ベクトルを取得
    fn direction_3d(&self) -> Self::Direction3D;
}
