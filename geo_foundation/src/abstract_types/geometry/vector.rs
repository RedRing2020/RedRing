//! Vector - ベクトルの抽象化トレイト
//!
//! CAD/CAM システムで使用されるベクトルの抽象化インターフェース

use std::fmt::Debug;
use std::ops::{Add, Sub, Mul, Index, IndexMut};

/// N次元ベクトルの抽象化トレイト
///
/// 次元数をコンパイル時定数として指定し、
/// 基本的なベクトル操作を抽象化する
pub trait Vector<const DIM: usize>: 
    Clone + Debug + PartialEq +
    Add<Output = Self> + Sub<Output = Self> + Mul<Self::Scalar, Output = Self> +
    Index<usize, Output = Self::Scalar> + IndexMut<usize>
{
    /// スカラー型（ベクトル成分の型）
    type Scalar: Copy + Debug + PartialEq + PartialOrd;

    /// 成分から新しいベクトルを作成
    fn from_components(components: [Self::Scalar; DIM]) -> Self
    where
        Self: Sized;

    /// 成分配列として取得
    fn components(&self) -> [Self::Scalar; DIM];

    /// ゼロベクトルを取得
    fn zero() -> Self
    where
        Self: Sized;

    /// 内積計算
    fn dot(&self, other: &Self) -> Self::Scalar;

    /// ベクトルの長さ（ノルム）
    fn length(&self) -> Self::Scalar;

    /// ベクトルの長さの二乗（計算効率化）
    fn length_squared(&self) -> Self::Scalar {
        self.dot(self)
    }

    /// 正規化（単位ベクトル化）
    fn normalize(&self) -> Option<Self>
    where
        Self: Sized;

    /// ゼロベクトルかどうかの判定
    fn is_zero(&self, tolerance: Self::Scalar) -> bool;

    /// 単位ベクトルかどうかの判定
    fn is_unit(&self, tolerance: Self::Scalar) -> bool;

    /// 他のベクトルと平行かどうかの判定
    fn is_parallel_to(&self, other: &Self, tolerance: Self::Scalar) -> bool;

    /// 他のベクトルと垂直かどうかの判定
    fn is_perpendicular_to(&self, other: &Self, tolerance: Self::Scalar) -> bool;
}

/// 2Dベクトルの追加機能
pub trait Vector2D: Vector<2> {
    /// X成分を取得
    fn x(&self) -> Self::Scalar;
    
    /// Y成分を取得
    fn y(&self) -> Self::Scalar;
    
    /// 成分から2Dベクトルを作成
    fn new(x: Self::Scalar, y: Self::Scalar) -> Self
    where
        Self: Sized;

    /// 90度回転したベクトルを取得（垂直ベクトル）
    fn perpendicular(&self) -> Self;

    /// 2D外積（スカラー値）
    fn cross_2d(&self, other: &Self) -> Self::Scalar;

    /// X軸の単位ベクトル
    fn unit_x() -> Self
    where
        Self: Sized;

    /// Y軸の単位ベクトル
    fn unit_y() -> Self
    where
        Self: Sized;
}

/// 3Dベクトルの追加機能
pub trait Vector3D: Vector<3> {
    /// X成分を取得
    fn x(&self) -> Self::Scalar;
    
    /// Y成分を取得
    fn y(&self) -> Self::Scalar;
    
    /// Z成分を取得
    fn z(&self) -> Self::Scalar;
    
    /// 成分から3Dベクトルを作成
    fn new(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Self
    where
        Self: Sized;

    /// 外積計算
    fn cross(&self, other: &Self) -> Self;

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

/// 2Dベクトル専用の拡張トレイト
pub trait Vector2DExt: Vector2D {
    /// 角度（ラジアン）から2Dベクトルを作成
    fn from_angle(angle: Self::Scalar) -> Self
    where
        Self: Sized;

    /// ベクトルの角度（ラジアン）を取得
    fn angle(&self) -> Self::Scalar;

    /// 他のベクトルとの角度差を取得
    fn angle_to(&self, other: &Self) -> Self::Scalar;
}

/// 3Dベクトル専用の拡張トレイト
pub trait Vector3DExt: Vector3D {
    /// 指定した軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self;

    /// 任意の軸に対する直交ベクトルを生成
    fn any_perpendicular(&self) -> Self;

    /// 正規直交基底を構築（このベクトルをZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self);
}

/// デフォルトの許容誤差
pub const DEFAULT_VECTOR_TOLERANCE: f64 = 1e-10;