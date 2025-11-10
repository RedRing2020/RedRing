//! 動的サイズベクトル
//!
//! 任意次元のベクトル演算を効率的に処理
//! 大規模数値計算や機械学習用途に適している
use crate::abstract_types::Scalar;
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

/// 動的サイズベクトル（高速演算用）
#[derive(Debug, Clone, PartialEq)]
pub struct Vector<T: Scalar> {
    data: Vec<T>,
}

impl<T: Scalar> Vector<T> {
    /// 新しいベクトルを作成
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }

    /// データから直接作成
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    /// ゼロベクトルを作成
    pub fn zeros(size: usize) -> Self {
        Self {
            data: vec![T::ZERO; size],
        }
    }

    /// 単位ベクトルを作成
    pub fn ones(size: usize) -> Self {
        Self {
            data: vec![T::ONE; size],
        }
    }

    /// 指定したインデックスが1で他が0のベクトル
    pub fn unit(size: usize, index: usize) -> Result<Self, String> {
        if index >= size {
            return Err("Index out of bounds".to_string());
        }
        let mut data = vec![T::ZERO; size];
        data[index] = T::ONE;
        Ok(Self { data })
    }

    /// ベクトルのサイズ
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 空かどうか
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 要素にアクセス
    pub fn get(&self, index: usize) -> T {
        self.data[index]
    }

    /// 要素を設定
    pub fn set(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> Result<T, String> {
        if self.len() != other.len() {
            return Err("Vector dimensions mismatch".to_string());
        }

        Ok(self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| *a * *b)
            .fold(T::ZERO, |acc, x| acc + x))
    }

    /// ユークリッドノルム（L2ノルム）
    pub fn norm(&self) -> T {
        self.data
            .iter()
            .map(|x| *x * *x)
            .fold(T::ZERO, |acc, x| acc + x)
            .sqrt()
    }

    /// L1ノルム
    pub fn norm_l1(&self) -> T {
        self.data
            .iter()
            .map(|x| x.abs())
            .fold(T::ZERO, |acc, x| acc + x)
    }

    /// 無限大ノルム
    pub fn norm_inf(&self) -> T {
        self.data
            .iter()
            .map(|x| x.abs())
            .fold(T::ZERO, |acc, x| acc.max(x))
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero vector".to_string());
        }
        Ok(Self {
            data: self.data.iter().map(|x| *x / norm).collect(),
        })
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self {
            data: self.data.iter().map(|x| *x * scalar).collect(),
        }
    }

    /// 要素ごとの積（Hadamard積）
    pub fn hadamard(&self, other: &Self) -> Result<Self, String> {
        if self.len() != other.len() {
            return Err("Vector dimensions mismatch".to_string());
        }

        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a * *b)
                .collect(),
        })
    }

    /// データへの不変参照
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// データへの可変参照（注意して使用）
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// ベクトルを拡張（要素を追加）
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// ベクトルを縮小（最後の要素を削除）
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    /// ベクトルをリサイズ
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.data.resize(new_len, value);
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Vector<T> {
    type Output = Result<Self, String>;

    fn add(self, other: Self) -> Self::Output {
        if self.len() != other.len() {
            return Err("Vector dimensions mismatch".to_string());
        }

        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a + *b)
                .collect(),
        })
    }
}

impl<T: Scalar> Sub for Vector<T> {
    type Output = Result<Self, String>;

    fn sub(self, other: Self) -> Self::Output {
        if self.len() != other.len() {
            return Err("Vector dimensions mismatch".to_string());
        }

        Ok(Self {
            data: self
                .data
                .iter()
                .zip(other.data.iter())
                .map(|(a, b)| *a - *b)
                .collect(),
        })
    }
}

impl<T: Scalar> Mul<T> for Vector<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        self.scale(scalar)
    }
}

impl<T: Scalar> Div<T> for Vector<T> {
    type Output = Self;

    fn div(self, scalar: T) -> Self::Output {
        Self {
            data: self.data.into_iter().map(|x| x / scalar).collect(),
        }
    }
}

impl<T: Scalar> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Scalar> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

/// 型エイリアス
pub type VectorF = Vector<f32>;
pub type VectorD = Vector<f64>;
