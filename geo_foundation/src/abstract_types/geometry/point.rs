//! Point - 点の抽象化トレイト
//!
//! CAD/CAM システムで使用される点の抽象化インターフェース

use crate::abstract_types::TolerantEq;
use std::fmt::Debug;

/// N次元点の抽象化トレイト
///
/// 次元数をコンパイル時定数として指定し、
/// 基本的な点操作を抽象化する
pub trait Point<const DIM: usize>: Clone + Debug + PartialEq + TolerantEq {
    /// スカラー型（座標値の型）
    type Scalar: Copy + Debug + PartialEq + PartialOrd;
    
    /// ベクトル型（点間の差分を表す）
    type Vector: Clone + Debug;
    
    /// 原点を取得
    fn origin() -> Self
    where
        Self: Sized;
    
    /// 点間距離を計算
    fn distance_to(&self, other: &Self) -> Self::Scalar;
    
    /// ベクトルで点を移動
    fn translate(&self, vector: &Self::Vector) -> Self;
    
    /// 2点間のベクトルを取得
    fn vector_to(&self, other: &Self) -> Self::Vector;
    
    /// 座標配列として取得（次元に応じた配列）
    fn coords(&self) -> [Self::Scalar; DIM];
}

/// 2D点の追加機能
pub trait Point2D: Point<2> {
    /// X座標を取得
    fn x(&self) -> Self::Scalar;
    
    /// Y座標を取得
    fn y(&self) -> Self::Scalar;
    
    /// 成分から点を作成
    fn from_components(x: Self::Scalar, y: Self::Scalar) -> Self
    where
        Self: Sized;
    
    /// 座標配列から点を作成
    fn from_coords(coords: [Self::Scalar; 2]) -> Self
    where
        Self: Sized
    {
        Self::from_components(coords[0], coords[1])
    }
}

/// 3D点の追加機能
pub trait Point3D: Point<3> {
    /// X座標を取得
    fn x(&self) -> Self::Scalar;
    
    /// Y座標を取得
    fn y(&self) -> Self::Scalar;
    
    /// Z座標を取得
    fn z(&self) -> Self::Scalar;
    
    /// 成分から点を作成
    fn from_components(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self
    where
        Self: Sized;
    
    /// 座標配列から点を作成
    fn from_coords(coords: [Self::Scalar; 3]) -> Self
    where
        Self: Sized
    {
        Self::from_components(coords[0], coords[1], coords[2])
    }
}
