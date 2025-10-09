//! Circle (円) トレイト定義
//!
//! 2D/3D空間における円の抽象的なインターフェースを提供

use crate::abstract_types::Scalar;
// use crate::constants::precision::{PI, TAU}; // 使用されていないため削除

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

// Arc トレイトは arc.rs で定義されています
// 重複を避けるため、ここからは削除しました
