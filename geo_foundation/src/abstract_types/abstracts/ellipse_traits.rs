//! Ellipse Traits - 楕円の最小責務抽象化
//!
//! abstracts 層での Ellipse 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 2D楕円の最小責務抽象化
///
/// 実装に依存しない純粋なインターフェース定義
pub trait Ellipse2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 楕円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;
}

/// 3D楕円の最小責務抽象化
pub trait Ellipse3D<T: Scalar>: Ellipse2D<T> {
    /// ベクトル型
    type Vector;

    /// 楕円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> Self::Vector;

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> Self::Vector;
}

/// 楕円の計量最小責務
pub trait EllipseMetrics<T: Scalar>: Ellipse2D<T> {
    /// 楕円の面積を計算
    fn area(&self) -> T;

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T;

    /// 楕円の離心率を計算
    fn eccentricity(&self) -> T;
}

/// 楕円の包含判定最小責務
pub trait EllipseContainment<T: Scalar>: Ellipse2D<T> {
    /// 点が楕円内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が楕円境界上にあるかを判定（許容誤差考慮）
    fn on_ellipse(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 点から楕円への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}

/// 楕円の形状判定最小責務
pub trait EllipseShape<T: Scalar>: Ellipse2D<T> {
    /// 楕円が円に近いかを判定
    fn is_nearly_circular(&self, tolerance: T) -> bool;

    /// 楕円が完全な円かを判定
    fn is_circle(&self) -> bool;
}