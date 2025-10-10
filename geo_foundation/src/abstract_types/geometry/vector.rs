//! Vector - ベクトルの最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本Vectorトレイト = ベクトル演算の最小限
//! ```text
//! Vector Trait = 基本属性・演算のみ
//! ├── 成分操作 (from_components, components, indexing)
//! ├── 基本演算 (zero, dot, length, normalize)
//! ├── 判定 (is_zero, is_unit, is_parallel, is_perpendicular)
//! └── 基本代数 (Add, Sub, Mul via operator traits)
//!
//! 除外される責務:
//! ├── 次元特化アクセス (x, y, z) → Vector2D/Vector3D
//! ├── 幾何演算 (cross, perpendicular) → VectorGeometry
//! ├── 角度操作 (from_angle, angle_to) → VectorAngular
//! └── 高度な変換 (rotate, reflect) → geo_algorithms
//! ```
//!
//! ## 拡張トレイト群による機能分離
//! ```text
//! Vector2D/Vector3D: 次元特化アクセス
//! Vector2DGeometry: 2D幾何演算 (perpendicular, cross_2d)
//! Vector3DGeometry: 3D幾何演算 (cross, rotate_around_axis)
//! VectorAngular: 角度関連操作
//! ```

use crate::Scalar;
use std::fmt::Debug;
use std::ops::{Add, Index, IndexMut, Mul, Sub};

/// N次元ベクトルの最小責務トレイト
///
/// 基本的なベクトル演算のみを提供。
/// 次元特化機能や幾何演算は拡張トレイトで分離。
pub trait Vector<T: Scalar, const DIM: usize>:
    Clone
    + Debug
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<T, Output = Self>
    + Index<usize, Output = T>
    + IndexMut<usize>
{
    /// 成分から新しいベクトルを作成
    fn from_components(components: [T; DIM]) -> Self
    where
        Self: Sized;

    /// 成分配列として取得
    fn components(&self) -> [T; DIM];

    /// ゼロベクトルを取得
    fn zero() -> Self
    where
        Self: Sized;

    /// 内積計算
    fn dot(&self, other: &Self) -> T;

    /// ベクトルの長さ（ノルム）
    fn length(&self) -> T;

    /// ベクトルの長さの二乗（計算効率化）
    fn length_squared(&self) -> T {
        self.dot(self)
    }

    /// 正規化（単位ベクトル化）
    fn normalize(&self) -> Option<Self>
    where
        Self: Sized;

    /// ゼロベクトルかどうかの判定
    fn is_zero(&self, tolerance: T) -> bool;

    /// 単位ベクトルかどうかの判定
    fn is_unit(&self, tolerance: T) -> bool;

    /// 他のベクトルと平行かどうかの判定
    fn is_parallel_to(&self, other: &Self, tolerance: T) -> bool;

    /// 他のベクトルと垂直かどうかの判定
    fn is_perpendicular_to(&self, other: &Self, tolerance: T) -> bool;
}

/// 2Dベクトルの座標アクセス拡張
///
/// 基本Vectorトレイトに2D特化の座標アクセスを追加。
pub trait Vector2D<T: Scalar>: Vector<T, 2> {
    /// X成分を取得
    fn x(&self) -> T;

    /// Y成分を取得
    fn y(&self) -> T;

    /// 成分から2Dベクトルを作成
    fn new(x: T, y: T) -> Self
    where
        Self: Sized;
}

/// 2Dベクトルの幾何演算拡張
///
/// 2D平面特有の幾何演算を提供。基本機能とは分離。
pub trait Vector2DGeometry<T: Scalar>: Vector2D<T> {
    /// 90度回転したベクトルを取得（垂直ベクトル）
    fn perpendicular(&self) -> Self;

    /// 2D外積（スカラー値）
    fn cross_2d(&self, other: &Self) -> T;
}

/// 2Dベクトルの定数提供
///
/// 単位ベクトルの定数を提供。
pub trait Vector2DConstants<T: Scalar>: Vector2D<T> {
    /// X軸の単位ベクトル
    fn unit_x() -> Self
    where
        Self: Sized;

    /// Y軸の単位ベクトル
    fn unit_y() -> Self
    where
        Self: Sized;
}

/// 3Dベクトルの座標アクセス拡張
///
/// 基本Vectorトレイトに3D特化の座標アクセスを追加。
pub trait Vector3D<T: Scalar>: Vector<T, 3> {
    /// X成分を取得
    fn x(&self) -> T;

    /// Y成分を取得
    fn y(&self) -> T;

    /// Z成分を取得
    fn z(&self) -> T;

    /// 成分から3Dベクトルを作成
    fn new(x: T, y: T, z: T) -> Self
    where
        Self: Sized;
}

/// 3Dベクトルの幾何演算拡張
///
/// 3D空間特有の幾何演算を提供。
pub trait Vector3DGeometry<T: Scalar>: Vector3D<T> {
    /// 外積計算
    fn cross(&self, other: &Self) -> Self;
}

/// 3Dベクトルの定数提供
///
/// 3D単位ベクトルの定数を提供。
pub trait Vector3DConstants<T: Scalar>: Vector3D<T> {
    /// X軸の単位ベクトル
    fn unit_x() -> Self
    where
        Self: Sized;

    /// Y軸の単位ベクトル
    fn unit_y() -> Self
    where
        Self: Sized;

    /// Z軸の単位ベクトル
    fn unit_z() -> Self
    where
        Self: Sized;
}

/// 2Dベクトルの角度演算拡張
///
/// 角度関連の演算機能を提供。角度が必要な場合のみ使用。
pub trait Vector2DAngular<T: Scalar>: Vector2D<T> {
    /// 角度（ラジアン）から2Dベクトルを作成
    fn from_angle(angle: T) -> Self
    where
        Self: Sized;

    /// ベクトルの角度（ラジアン）を取得
    fn angle(&self) -> T;

    /// 他のベクトルとの角度差を取得
    fn angle_to(&self, other: &Self) -> T;
}

/// 3Dベクトルの回転演算拡張
///
/// 3D空間での回転操作を提供。
pub trait Vector3DRotation<T: Scalar>: Vector3D<T> {
    /// 指定した軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: T) -> Self;

    /// 任意の軸に対する直交ベクトルを生成
    fn any_perpendicular(&self) -> Self;

    /// 正規直交基底を構築（このベクトルをZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self);
}
