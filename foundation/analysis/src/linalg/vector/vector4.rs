//! 4次元ベクトル（同次座標系）
//!
//! 4x4変換行列との演算、同次座標系での3D変換に使用
//! 透視投影やアフィン変換での座標計算に最適化
use crate::abstract_types::Scalar;
use std::ops::{Add, Mul, Neg, Sub};

/// 4次元固定サイズベクトル（同次座標系）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector4<T: Scalar> {
    pub data: [T; 4],
}

impl<T: Scalar> Vector4<T> {
    /// X軸単位ベクトル定数
    pub const X_AXIS: Vector4<f64> = Vector4 {
        data: [1.0, 0.0, 0.0, 0.0],
    };

    /// Y軸単位ベクトル定数
    pub const Y_AXIS: Vector4<f64> = Vector4 {
        data: [0.0, 1.0, 0.0, 0.0],
    };

    /// Z軸単位ベクトル定数
    pub const Z_AXIS: Vector4<f64> = Vector4 {
        data: [0.0, 0.0, 1.0, 0.0],
    };

    /// W軸単位ベクトル定数
    pub const W_AXIS: Vector4<f64> = Vector4 {
        data: [0.0, 0.0, 0.0, 1.0],
    };

    /// ゼロベクトル定数
    pub const ZERO: Vector4<f64> = Vector4 {
        data: [0.0, 0.0, 0.0, 0.0],
    };

    /// 新しい4Dベクトルを作成
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { data: [x, y, z, w] }
    }

    /// ゼロベクトル - ZERO定数のエイリアス
    pub fn zero() -> Self {
        // 型変換を通じてZERO定数を任意のScalar型で利用可能にする
        Self::new(
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
        )
    }

    /// 3D点から同次座標へ変換（w=1）
    pub fn from_point(x: T, y: T, z: T) -> Self {
        Self::new(x, y, z, T::ONE)
    }

    /// 3D方向ベクトルから同次座標へ変換（w=0）
    pub fn from_direction(x: T, y: T, z: T) -> Self {
        Self::new(x, y, z, T::ZERO)
    }

    /// X軸方向の単位ベクトル（1, 0, 0, 0）- X_AXIS定数のエイリアス
    pub fn x_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(
            T::from_f64(1.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
        )
    }

    /// Y軸方向の単位ベクトル（0, 1, 0, 0）- Y_AXIS定数のエイリアス
    pub fn y_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(
            T::from_f64(0.0),
            T::from_f64(1.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
        )
    }

    /// Z軸方向の単位ベクトル（0, 0, 1, 0）- Z_AXIS定数のエイリアス
    pub fn z_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(1.0),
            T::from_f64(0.0),
        )
    }

    /// W軸方向の単位ベクトル（0, 0, 0, 1）- W_AXIS定数のエイリアス
    pub fn w_axis() -> Self {
        // 型変換を通じて定数を任意のScalar型で利用可能にする
        Self::new(
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(0.0),
            T::from_f64(1.0),
        )
    }

    /// X成分にアクセス
    pub fn x(&self) -> T {
        self.data[0]
    }

    /// Y成分にアクセス
    pub fn y(&self) -> T {
        self.data[1]
    }

    /// Z成分にアクセス
    pub fn z(&self) -> T {
        self.data[2]
    }

    /// W成分にアクセス
    pub fn w(&self) -> T {
        self.data[3]
    }

    /// 成分を設定
    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }

    pub fn set_z(&mut self, z: T) {
        self.data[2] = z;
    }

    pub fn set_w(&mut self, w: T) {
        self.data[3] = w;
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0]
            + self.data[1] * other.data[1]
            + self.data[2] * other.data[2]
            + self.data[3] * other.data[3]
    }

    /// ユークリッドノルム
    pub fn norm(&self) -> T {
        (self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]
            + self.data[3] * self.data[3])
            .sqrt()
    }

    /// ノルムの2乗（平方根計算を避ける）
    pub fn norm_squared(&self) -> T {
        self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]
            + self.data[3] * self.data[3]
    }

    /// 正規化（単位ベクトル化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero vector".to_string());
        }
        Ok(Self::new(
            self.data[0] / norm,
            self.data[1] / norm,
            self.data[2] / norm,
            self.data[3] / norm,
        ))
    }

    /// 同次座標からユークリッド座標への変換（w除法）
    pub fn to_euclidean(&self) -> Result<super::vector3::Vector3<T>, String> {
        if self.data[3].is_zero() {
            return Err("Cannot convert to euclidean: w component is zero".to_string());
        }
        Ok(super::vector3::Vector3::new(
            self.data[0] / self.data[3],
            self.data[1] / self.data[3],
            self.data[2] / self.data[3],
        ))
    }

    /// ユークリッド座標から同次座標への変換（静的メソッド）
    pub fn from_euclidean(v: super::vector3::Vector3<T>) -> Self {
        Self::new(v.x(), v.y(), v.z(), T::ONE)
    }

    /// 3次元部分を取得（w成分を無視）
    pub fn xyz(&self) -> super::vector3::Vector3<T> {
        super::vector3::Vector3::new(self.data[0], self.data[1], self.data[2])
    }

    /// 同次座標が点を表すかチェック（w != 0）
    pub fn is_point(&self) -> bool {
        !self.data[3].is_zero()
    }

    /// 同次座標が方向ベクトルを表すかチェック（w == 0）
    pub fn is_direction(&self) -> bool {
        self.data[3].is_zero()
    }

    /// 透視除法後の有効性をチェック
    pub fn is_valid_homogeneous(&self) -> bool {
        // w が非常に小さい場合は無効とみなす
        self.data[3].abs() > T::EPSILON
    }

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        *self * (T::ONE - t) + *other * t
    }

    /// スカラー倍
    pub fn scale(&self, scalar: T) -> Self {
        Self::new(
            self.data[0] * scalar,
            self.data[1] * scalar,
            self.data[2] * scalar,
            self.data[3] * scalar,
        )
    }

    /// 要素ごとの積（Hadamard積）
    pub fn hadamard(&self, other: &Self) -> Self {
        Self::new(
            self.data[0] * other.data[0],
            self.data[1] * other.data[1],
            self.data[2] * other.data[2],
            self.data[3] * other.data[3],
        )
    }

    /// 要素ごとの最小値
    pub fn min(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].min(other.data[0]),
            self.data[1].min(other.data[1]),
            self.data[2].min(other.data[2]),
            self.data[3].min(other.data[3]),
        )
    }

    /// 要素ごとの最大値
    pub fn max(&self, other: &Self) -> Self {
        Self::new(
            self.data[0].max(other.data[0]),
            self.data[1].max(other.data[1]),
            self.data[2].max(other.data[2]),
            self.data[3].max(other.data[3]),
        )
    }

    /// 絶対値
    pub fn abs(&self) -> Self {
        Self::new(
            self.data[0].abs(),
            self.data[1].abs(),
            self.data[2].abs(),
            self.data[3].abs(),
        )
    }

    /// クリッピング座標系での深度値を取得
    pub fn depth(&self) -> T {
        if self.data[3].is_zero() {
            T::ZERO
        } else {
            self.data[2] / self.data[3]
        }
    }

    /// NDC（正規化デバイス座標）への変換
    pub fn to_ndc(&self) -> Result<super::vector3::Vector3<T>, String> {
        if self.data[3].is_zero() {
            return Err("Cannot convert to NDC: w component is zero".to_string());
        }
        let w_inv = T::ONE / self.data[3];
        Ok(super::vector3::Vector3::new(
            self.data[0] * w_inv,
            self.data[1] * w_inv,
            self.data[2] * w_inv,
        ))
    }
}

// 演算子オーバーロード
impl<T: Scalar> Add for Vector4<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0] + other.data[0],
            self.data[1] + other.data[1],
            self.data[2] + other.data[2],
            self.data[3] + other.data[3],
        )
    }
}

impl<T: Scalar> Sub for Vector4<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self::new(
            self.data[0] - other.data[0],
            self.data[1] - other.data[1],
            self.data[2] - other.data[2],
            self.data[3] - other.data[3],
        )
    }
}

impl<T: Scalar> Mul<T> for Vector4<T> {
    type Output = Self;
    fn mul(self, scalar: T) -> Self::Output {
        Self::new(
            self.data[0] * scalar,
            self.data[1] * scalar,
            self.data[2] * scalar,
            self.data[3] * scalar,
        )
    }
}

impl<T: Scalar> Neg for Vector4<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.data[0], -self.data[1], -self.data[2], -self.data[3])
    }
}

/// 型エイリアス
pub type Vector4f = Vector4<f32>;
pub type Vector4d = Vector4<f64>;
