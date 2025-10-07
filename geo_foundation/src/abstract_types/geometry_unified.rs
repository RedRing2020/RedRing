//! 幾何学的要素の抽象化トレイト (統合されたバージョン)
//!
//! CAD/CAM システムで使用される基本的な幾何学要素の
//! 抽象化インターフェースを定義

// この統合バージョンは、個別ファイル移行後に削除予定
use crate::abstract_types::TolerantEq;
use std::fmt::Debug;

/// N次元ベクトルの抽象化
pub trait Vector<const DIM: usize>: Clone + Debug + PartialEq + TolerantEq {
    type Scalar: Copy + Debug + PartialEq + PartialOrd;

    /// ゼロベクトル
    fn zero() -> Self;

    /// ベクトルのノルム（長さ）
    fn norm(&self) -> Self::Scalar;

    /// ベクトルの正規化
    fn normalize(&self) -> Self;

    /// 内積計算
    fn dot(&self, other: &Self) -> Self::Scalar;

    /// スカラー倍
    fn scale(&self, scalar: Self::Scalar) -> Self;

    /// ベクトル加算
    fn add(&self, other: &Self) -> Self;

    /// ベクトル減算
    fn sub(&self, other: &Self) -> Self;
}

/// N次元点の抽象化
pub trait Point<const DIM: usize>: Clone + Debug + PartialEq + TolerantEq {
    type Scalar: Copy + Debug + PartialEq + PartialOrd;
    type Vector: Vector<DIM, Scalar = Self::Scalar>;

    /// 原点
    fn origin() -> Self;

    /// 点間距離
    fn distance_to(&self, other: &Self) -> Self::Scalar;

    /// ベクトル加算（点の移動）
    fn translate(&self, vector: &Self::Vector) -> Self;

    /// 点間ベクトル
    fn vector_to(&self, other: &Self) -> Self::Vector;
}

/// N次元方向ベクトルの抽象化
pub trait Direction<const DIM: usize>: Clone + Debug + PartialEq + TolerantEq {
    type Scalar: Copy + Debug + PartialEq + PartialOrd;
    type Vector: Vector<DIM, Scalar = Self::Scalar>;

    /// ベクトルから方向を作成（正規化）
    fn from_vector(vector: Self::Vector) -> Option<Self>;

    /// 方向をベクトルとして取得
    fn to_vector(&self) -> Self::Vector;

    /// 内積計算
    fn dot(&self, other: &Self) -> Self::Scalar;

    /// 方向の反転
    fn reverse(&self) -> Self;

    /// 平行性判定
    fn is_parallel(&self, other: &Self) -> bool;

    /// 垂直性判定
    fn is_perpendicular(&self, other: &Self) -> bool;
}

/// 境界ボックスの抽象化
pub trait BoundingBox<const DIM: usize>: Clone + Debug + PartialEq {
    type Scalar: Copy + Debug + PartialEq + PartialOrd;
    type Point: Point<DIM, Scalar = Self::Scalar>;

    /// 最小点
    fn min(&self) -> &Self::Point;

    /// 最大点
    fn max(&self) -> &Self::Point;

    /// 点が境界内にあるか判定
    fn contains(&self, point: &Self::Point) -> bool;

    /// 境界ボックス同士の交差判定
    fn intersects(&self, other: &Self) -> bool;

    /// 境界ボックスの結合
    fn union(&self, other: &Self) -> Self;
}

/// 2D専用の追加機能
pub trait Vector2DExt: Vector<2> {
    /// 垂直ベクトル（90度回転）
    fn perpendicular(&self) -> Self;

    /// 外積のZ成分（2Dでの回転方向判定）
    fn cross_z(&self, other: &Self) -> Self::Scalar;
}

/// 3D専用の追加機能
pub trait Vector3DExt: Vector<3> {
    /// 外積計算
    fn cross(&self, other: &Self) -> Self;
}

/// 2D方向ベクトルの追加機能
pub trait Direction2DExt: Direction<2> {
    /// 角度から方向を作成
    fn from_angle(angle: Self::Scalar) -> Self;

    /// 方向の角度を取得
    fn to_angle(&self) -> Self::Scalar;

    /// 垂直方向
    fn perpendicular(&self) -> Self;
}

/// 3D方向ベクトルの追加機能
pub trait Direction3DExt: Direction<3> {
    /// 外積計算
    fn cross(&self, other: &Self) -> Self::Vector;

    /// 軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self;

    /// 任意の垂直ベクトルを生成
    fn any_perpendicular(&self) -> Self;
}