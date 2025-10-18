//! 2次元ベクトル
//!
//! 2D幾何計算、グラフィックス、UI座標に最適化
//! 高速な演算のためコンパイル時サイズ確定
use crate::abstract_types::Scalar;
use std::ops::{Add, Mul, Neg, Sub};

/// 2次元固定サイズベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2<T: Scalar> {
    pub data: [T; 2],
}

impl<T: Scalar> Vector2<T> {
    /// X軸単位ベクトル定数
    pub const X_AXIS: Vector2<f64> = Vector2 { data: [1.0, 0.0] };

    /// Y軸単位ベクトル定数
    pub const Y_AXIS: Vector2<f64> = Vector2 { data: [0.0, 1.0] };

    /// ゼロベクトル定数
    pub const ZERO: Vector2<f64> = Vector2 { data: [0.0, 0.0] };

    /// 新しい2Dベクトルを作成
    pub fn new(x: T, y: T) -> Self {
        Self { data: [x, y] }
    }

    /// ゼロベクトル - ZERO定数のエイリアス
    pub fn zero() -> Self {
        // 型変換を通じてZERO定数を任意のScalar型で利用可能にする
        Self::new(T::from_f64(0.0), T::from_f64(0.0))
    }

    /// 単位ベクトル（1, 1）正規化済み
    pub fn one() -> Self {
        let sqrt2_inv = T::ONE / (T::ONE + T::ONE).sqrt();
        Self::new(sqrt2_inv, sqrt2_inv)
    }

    /// X軸方向の単位ベクトル（1, 0）- X_AXIS定数のエイリアス
    pub fn x_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(T::from_f64(1.0), T::from_f64(0.0))
    }

    /// Y軸方向の単位ベクトル（0, 1）- Y_AXIS定数のエイリアス
    pub fn y_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(T::from_f64(0.0), T::from_f64(1.0))
    }

    /// X成分にアクセス
    pub fn x(&self) -> T {
        self.data[0]
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T {
        self.data[1]
    }

    /// 成分を設定
    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0] + self.data[1] * other.data[1]
    }

    /// 外積（スカラー値）2Dでは z 成分のみ
    pub fn cross(&self, other: &Self) -> T {
        self.data[0] * other.data[1] - self.data[1] * other.data[0]
    }

    /// ユークリッドノルム
    pub fn norm(&self) -> T {
        (self.data[0] * self.data[0] + self.data[1] * self.data[1]).sqrt()
    }

    /// ノルムの2乗（平方根計算を避ける）
    pub fn norm_squared(&self) -> T {
        self.data[0] * self.data[0] + self.data[1] * self.data[1]
    }

    /// 他のベクトルとのマンハッタン距離（metrics::manhattan_distanceのラッパー）
    pub fn manhattan_distance_to(&self, other: &Self) -> T {
        crate::metrics::manhattan_distance(&[self.x(), self.y()], &[other.x(), other.y()])
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero vector".to_string());
        }
        Ok(Self::new(self.data[0] / norm, self.data[1] / norm))
    }

    /// 90度回転（反時計回り）
    pub fn rotate_90(&self) -> Self {
        Self::new(-self.data[1], self.data[0])
    }

    /// 任意角度回転（ラジアン、反時計回り）
    pub fn rotate(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Self::new(
            self.data[0] * cos_a - self.data[1] * sin_a,
            self.data[0] * sin_a + self.data[1] * cos_a,
        )
    }

    /// 他のベクトルとの角度（ラジアン）
    pub fn angle_to(&self, other: &Self) -> T {
        let dot = self.dot(other);
        let cross = self.cross(other);
        cross.atan2(dot)
    }

    /// 原点からの角度（ラジアン）
    pub fn angle(&self) -> T {
        self.data[1].atan2(self.data[0])
    }

    /// 直交ベクトル（時計回り90度回転）
    pub fn perpendicular(&self) -> Self {
        Self::new(self.data[1], -self.data[0])
    }

    /// 他のベクトルへの射影
    pub fn project_onto(&self, other: &Self) -> Result<Self, String> {
        let other_norm_sq = other.norm_squared();
        if other_norm_sq.is_zero() {
            return Err("Cannot project onto zero vector".to_string());
        }
        let scalar = self.dot(other) / other_norm_sq;
        Ok(*other * scalar)
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self * (T::ONE - t) + *other * t
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(self.data[0] * scalar, self.data[1] * scalar)
    }

    /// 要素ごとの積（Hadamard積）
    pub fn hadamard(&self, other: &Self) -> Self {
        Self::new(self.data[0] * other.data[0], self.data[1] * other.data[1])
    }

    /// 要素ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].min(other.data[0]),
            self.data[1].min(other.data[1]),
        )
    }

    /// 要素ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].max(other.data[0]),
            self.data[1].max(other.data[1]),
        )
    }

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(self.data[0].abs(), self.data[1].abs())
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Vector2<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(self.data[0] + other.data[0], self.data[1] + other.data[1])
    }
}

impl<T: Scalar> Sub for Vector2<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.data[0] - other.data[0], self.data[1] - other.data[1])
    }
}

impl<T: Scalar> Mul<T> for Vector2<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.data[0] * scalar, self.data[1] * scalar)
    }
}

impl<T: Scalar> Neg for Vector2<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.data[0], -self.data[1])
    }
}

/// 型エイリアス
pub type Vector2f = Vector2<f32>;
pub type Vector2d = Vector2<f64>;
