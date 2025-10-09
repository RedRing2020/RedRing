//! Circle (円) トレイト定義
//!
//! 2D/3D空間における円の抽象的なインターフェースを提供

use crate::abstract_types::Scalar;
use crate::constants::precision::{PI, TAU};

/// 2D円の基本操作を定義するトレイト
pub trait Circle2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;
}

/// 3D円の基本操作を定義するトレイト
pub trait Circle3D<T: Scalar>: Circle2D<T> {
    /// ベクトル型の定義
    type Vector;

    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;
}

/// 円弧（Arc）の基本操作を定義するトレイト
pub trait Arc2D<T: Scalar>: Circle2D<T> {
    /// 円弧の開始角度を取得
    fn start_angle(&self) -> T;

    /// 円弧の終了角度を取得
    fn end_angle(&self) -> T;

    /// 円弧の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 円弧の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 円弧上の指定されたパラメータ（0.0〜1.0）での点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;
}

/// 3D円弧の基本操作を定義するトレイト
pub trait Arc3D<T: Scalar>: Arc2D<T> + Circle3D<T> {
    // 3D固有の円弧操作は現在なし
    // 必要に応じて追加
}
