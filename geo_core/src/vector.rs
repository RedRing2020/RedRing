/// ベクトル演算システム
/// 
/// 統合された2D/3Dベクトル演算を提供し、許容誤差を考慮した
/// 堅牢な幾何計算を実現する。

use crate::scalar::Scalar;
use crate::tolerance::{ToleranceContext, TolerantEq};
use std::fmt;
use std::ops::{Add, Sub, Mul, Index, IndexMut};

/// 汎用ベクトルトレイト
pub trait Vector<const D: usize>:
    Clone + PartialEq + fmt::Debug + fmt::Display
    + Add<Output = Self> + Sub<Output = Self> + Mul<Scalar, Output = Self>
    + TolerantEq + Index<usize, Output = Scalar> + IndexMut<usize>
{
    /// 成分から新しいベクトルを作成
    fn new(components: [Scalar; D]) -> Self;

    /// 成分配列への参照を取得
    fn components(&self) -> &[Scalar; D];

    /// 可変成分配列への参照を取得
    fn components_mut(&mut self) -> &mut [Scalar; D];

    /// 次元数を取得
    fn dimension() -> usize { D }

    /// 内積
    fn dot(&self, other: &Self) -> Scalar;

    /// ノルム（長さ）
    fn norm(&self) -> Scalar;

    /// ノルムの2乗（計算効率化のため）
    fn norm_squared(&self) -> Scalar {
        self.dot(self)
    }

    /// 正規化（単位ベクトル化）
    fn normalize(&self, context: &ToleranceContext) -> Option<Self>;

    /// ゼロベクトルかどうかの判定
    fn is_zero(&self, context: &ToleranceContext) -> bool {
        self.norm().value() < context.linear
    }

    /// 単位ベクトルかどうかの判定
    fn is_unit(&self, context: &ToleranceContext) -> bool {
        (self.norm().value() - 1.0).abs() < context.linear
    }

    /// 他のベクトルと平行かどうかの判定
    fn is_parallel_to(&self, other: &Self, context: &ToleranceContext) -> bool;

    /// 他のベクトルと垂直かどうかの判定
    fn is_perpendicular_to(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.dot(other).tolerant_eq(&Scalar::new(0.0), context)
    }

    /// 成分ごとの最小値
    fn component_min(&self, other: &Self) -> Self;

    /// 成分ごとの最大値
    fn component_max(&self, other: &Self) -> Self;

    /// 成分ごとの絶対値
    fn abs(&self) -> Self;
}

// サブモジュールをインクルード
#[path = "vector2d.rs"]
pub mod vector2d;

#[path = "vector3d.rs"]
pub mod vector3d;

// 型の再エクスポート
pub use vector2d::Vector2D;
pub use vector3d::{Vector3D, Direction3D};