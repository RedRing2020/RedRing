//! Circle Traits - 円の最小責務抽象化
//!
//! abstracts 層での Circle 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 円の最小責務抽象化
///
/// 実装に依存しない純粋なインターフェース定義
pub trait Circle2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;
}

/// 3D円の最小責務抽象化
pub trait Circle3D<T: Scalar>: Circle2D<T> {
    /// ベクトル型
    type Vector;

    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;
}

/// 円の計量最小責務
pub trait CircleMetrics<T: Scalar>: Circle2D<T> {
    /// 円の面積を計算
    fn area(&self) -> T;

    /// 円の周長を計算
    fn circumference(&self) -> T;
}

/// 円の包含判定最小責務
pub trait CircleContainment<T: Scalar>: Circle2D<T> {
    /// 点が円内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が円周上にあるかを判定（許容誤差考慮）
    fn on_circle(&self, point: &Self::Point, tolerance: T) -> bool;
}
