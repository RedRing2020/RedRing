//! Point - 点の最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本Pointトレイト = 座標アクセスと基本操作のみ
//! ```text
//! Point Trait = 基本属性・操作のみ
//! ├── 座標アクセス (coords)
//! ├── 距離計算 (distance_to)
//! ├── ベクトル変換 (vector_to, translate)
//! └── 原点生成 (origin)
//!
//! 除外される責務:
//! ├── 変換操作 (rotate, scale, reflect) → PointTransform
//! ├── 幾何判定 (inside_polygon, on_line) → geo_algorithms
//! ├── 集合操作 (centroid, convex_hull) → geo_algorithms
//! └── 空間インデックス (morton_code, hilbert) → geo_algorithms
//! ```
//!
//! ## 次元特化拡張によるアクセサ分離
//! ```text
//! Point2D: 2D座標アクセス (x, y)
//! Point3D: 3D座標アクセス (x, y, z)
//! PointND: N次元座標アクセス (component)
//! ```

use crate::{Scalar, TolerantEq};
use std::fmt::Debug;

/// N次元点の最小責務トレイト
///
/// 座標アクセス、距離計算、基本変換のみを提供。
/// 次元特化機能や複雑な幾何操作は拡張トレイトで分離。
pub trait Point<T: Scalar, const DIM: usize>: Clone + Debug + PartialEq + TolerantEq {
    /// ベクトル型（点間の差分を表す）
    type Vector: Clone + Debug;

    /// 原点を取得
    fn origin() -> Self
    where
        Self: Sized;

    /// 点間距離を計算
    fn distance_to(&self, other: &Self) -> T;

    /// ベクトルで点を移動
    fn translate(&self, vector: &Self::Vector) -> Self;

    /// 2点間のベクトルを取得
    fn vector_to(&self, other: &Self) -> Self::Vector;

    /// 座標配列として取得（次元に応じた配列）
    fn coords(&self) -> [T; DIM];
}

/// 2D点の座標アクセス拡張
///
/// 基本Pointトレイトに2D特化の座標アクセスを追加。
/// 座標軸アクセスのみに責務を限定。
pub trait Point2D<T: Scalar>: Point<T, 2> {
    /// X座標を取得
    fn x(&self) -> T;

    /// Y座標を取得
    fn y(&self) -> T;

    /// 成分から点を作成
    fn from_components(x: T, y: T) -> Self
    where
        Self: Sized;

    /// 座標配列から点を作成
    fn from_coords(coords: [T; 2]) -> Self
    where
        Self: Sized,
    {
        Self::from_components(coords[0], coords[1])
    }
}

/// 3D点の座標アクセス拡張
///
/// 基本Pointトレイトに3D特化の座標アクセスを追加。
pub trait Point3D<T: Scalar>: Point<T, 3> {
    /// X座標を取得
    fn x(&self) -> T;

    /// Y座標を取得
    fn y(&self) -> T;

    /// Z座標を取得
    fn z(&self) -> T;

    /// 成分から点を作成
    fn from_components(x: T, y: T, z: T) -> Self
    where
        Self: Sized;

    /// 座標配列から点を作成
    fn from_coords(coords: [T; 3]) -> Self
    where
        Self: Sized,
    {
        Self::from_components(coords[0], coords[1], coords[2])
    }
}
