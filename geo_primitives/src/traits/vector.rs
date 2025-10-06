/// ベクトル演算システム
///
/// 統合された2D/3Dベクトル演算を提供し、f64ベースの
/// 堅牢な幾何計算を実現する。

use std::fmt;
use std::ops::{Add, Sub, Mul, Index, IndexMut};

/// 汎用ベクトルトレイト（f64ベース）
pub trait Vector<const D: usize>:
    Clone + PartialEq + fmt::Debug + fmt::Display
    + Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self>
    + Index<usize, Output = f64> + IndexMut<usize>
{
    /// 成分から新しいベクトルを作成
    fn new(components: [f64; D]) -> Self;

    /// 成分配列への参照を取得
    fn components(&self) -> &[f64; D];

    /// 可変成分配列への参照を取得
    fn components_mut(&mut self) -> &mut [f64; D];

    /// 次元数を取得
    fn dimension() -> usize { D }

    /// 内積
    fn dot(&self, other: &Self) -> f64;

    /// ノルム（長さ）
    fn norm(&self) -> f64;

    /// ノルムの2乗（計算効率化のため）
    fn norm_squared(&self) -> f64 {
        self.dot(self)
    }

    /// 正規化（単位ベクトル化）
    fn normalize(&self) -> Option<Self>;

    /// ゼロベクトルかどうかの判定
    fn is_zero(&self, tolerance: f64) -> bool {
        self.norm() < tolerance
    }

    /// 単位ベクトルかどうかの判定
    fn is_unit(&self, tolerance: f64) -> bool {
        (self.norm() - 1.0).abs() < tolerance
    }

    /// 他のベクトルと平行かどうかの判定
    fn is_parallel_to(&self, other: &Self, tolerance: f64) -> bool;

    /// 他のベクトルと垂直かどうかの判定
    fn is_perpendicular_to(&self, other: &Self, tolerance: f64) -> bool {
        self.dot(other).abs() < tolerance
    }

    /// 成分ごとの最小値
    fn component_min(&self, other: &Self) -> Self;

    /// 成分ごとの最大値
    fn component_max(&self, other: &Self) -> Self;

    /// 成分ごとの絶対値
    fn abs(&self) -> Self;

    /// スカラー倍
    fn scale(&self, scalar: f64) -> Self;

    /// ゼロベクトル
    fn zero() -> Self;
}

/// デフォルトの許容誤差
pub const DEFAULT_VECTOR_TOLERANCE: f64 = 1e-10;
