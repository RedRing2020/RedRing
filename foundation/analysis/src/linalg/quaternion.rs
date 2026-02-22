//! クォータニオン（四元数）
//!
//! 3D回転の表現に特化したクォータニオン実装
//! - ジンバルロック回避
//! - 効率的な回転合成
//! - 滑らかな補間（SLERP）
//! - 単位クォータニオンによる回転表現
use crate::abstract_types::Scalar;
use crate::linalg::vector::{Vector3, Vector4};
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

/// 四元数計算用閾値
mod thresholds {
    /// ベクトル垂直判定閾値
    ///
    /// x成分の絶対値がこの値未満の場合、x軸と垂直と見なす。
    /// from_to_rotation() で反対方向ベクトルの回転軸を求める際に使用。
    pub const PERPENDICULAR_THRESHOLD: f64 = 0.9;

    /// SLERP閾値
    ///
    /// 内積がこの値以上の場合、線形補間（LERP）を使用する。
    /// 角度が非常に小さい場合の数値安定性を向上させる。
    pub const SLERP_THRESHOLD: f64 = 0.9995;
}

/// 単位クォータニオン（回転表現用）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion<T: Scalar> {
    /// クォータニオンの成分 [x, y, z, w] = [i, j, k, real]
    /// w: 実部（スカラー部）
    /// x, y, z: 虚部（ベクトル部）
    data: [T; 4],
}

impl<T: Scalar> Quaternion<T> {
    // === 定数 ===

    /// 単位クォータニオン（回転なし）
    pub const IDENTITY: Quaternion<f64> = Quaternion {
        data: [0.0, 0.0, 0.0, 1.0],
    };

    /// ゼロクォータニオン
    pub const ZERO: Quaternion<f64> = Quaternion {
        data: [0.0, 0.0, 0.0, 0.0],
    };

    // === コンストラクタ ===

    /// 新しいクォータニオンを作成 (x, y, z, w)
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { data: [x, y, z, w] }
    }

    /// 単位クォータニオンを作成（回転なし）
    pub fn identity() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO, T::ONE)
    }

    /// ゼロクォータニオンを作成
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO, T::ZERO)
    }

    /// 軸と角度から回転クォータニオンを作成
    /// axis: 正規化された回転軸
    /// angle: 回転角度（ラジアン）
    pub fn from_axis_angle(axis: &Vector3<T>, angle: T) -> Self {
        let half_angle = angle / T::from_f64(2.0);
        let sin_half = half_angle.sin();
        let cos_half = half_angle.cos();

        Self::new(
            axis.x() * sin_half,
            axis.y() * sin_half,
            axis.z() * sin_half,
            cos_half,
        )
    }

    /// オイラー角からクォータニオンを作成（XYZ順）
    /// angles: (pitch, yaw, roll) in radians
    pub fn from_euler_angles(pitch: T, yaw: T, roll: T) -> Self {
        let half_pitch = pitch / T::from_f64(2.0);
        let half_yaw = yaw / T::from_f64(2.0);
        let half_roll = roll / T::from_f64(2.0);

        let cp = half_pitch.cos();
        let sp = half_pitch.sin();
        let cy = half_yaw.cos();
        let sy = half_yaw.sin();
        let cr = half_roll.cos();
        let sr = half_roll.sin();

        Self::new(
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
            cr * cp * cy + sr * sp * sy,
        )
    }

    /// 2つのベクトル間の回転を表すクォータニオンを作成
    pub fn from_to_rotation(from: &Vector3<T>, to: &Vector3<T>) -> Result<Self, String> {
        let from_normalized = from.normalize()?;
        let to_normalized = to.normalize()?;

        let dot = from_normalized.dot(&to_normalized);

        // ベクトルが同じ方向の場合
        if dot >= T::ONE - T::EPSILON {
            return Ok(Self::identity());
        }

        // ベクトルが反対方向の場合
        if dot <= -T::ONE + T::EPSILON {
            // 垂直なベクトルを見つける
            let axis =
                if from_normalized.x().abs() < T::from_f64(thresholds::PERPENDICULAR_THRESHOLD) {
                    Vector3::new(T::ONE, T::ZERO, T::ZERO).cross(&from_normalized)
                } else {
                    Vector3::new(T::ZERO, T::ONE, T::ZERO).cross(&from_normalized)
                };
            let normalized_axis = axis.normalize()?;
            return Ok(Self::from_axis_angle(&normalized_axis, T::PI));
        }

        let cross = from_normalized.cross(&to_normalized);
        let w = T::ONE + dot;

        Self::new(cross.x(), cross.y(), cross.z(), w).normalize()
    }

    /// Vector4からクォータニオンを作成
    pub fn from_vector4(v: Vector4<T>) -> Self {
        Self::new(v.x(), v.y(), v.z(), v.w())
    }

    // === アクセサ ===

    /// X成分（i）を取得
    pub fn x(&self) -> T {
        self.data[0]
    }

    /// Y成分（j）を取得
    pub fn y(&self) -> T {
        self.data[1]
    }

    /// Z成分（k）を取得
    pub fn z(&self) -> T {
        self.data[2]
    }

    /// W成分（実部）を取得
    pub fn w(&self) -> T {
        self.data[3]
    }

    /// X成分を設定
    pub fn set_x(&mut self, x: T) {
        self.data[0] = x;
    }

    /// Y成分を設定
    pub fn set_y(&mut self, y: T) {
        self.data[1] = y;
    }

    /// Z成分を設定
    pub fn set_z(&mut self, z: T) {
        self.data[2] = z;
    }

    /// W成分を設定
    pub fn set_w(&mut self, w: T) {
        self.data[3] = w;
    }

    // === 汎用アクセサ ===

    /// インデックスで要素を取得
    #[inline]
    pub fn get(&self, index: usize) -> T {
        self.data[index]
    }

    /// インデックスで要素を設定
    #[inline]
    pub fn set(&mut self, index: usize, value: T) {
        self.data[index] = value;
    }

    /// 内部配列への参照
    #[inline]
    pub fn as_array(&self) -> &[T; 4] {
        &self.data
    }

    // === イテレータ ===

    /// 全要素をイテレート
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.data.iter().copied()
    }

    // === ベクトル部分・スカラー部分 ===

    /// 虚部ベクトル (x, y, z) を取得
    pub fn vector_part(&self) -> Vector3<T> {
        Vector3::new(self.data[0], self.data[1], self.data[2])
    }

    /// スカラー部 (w) を取得
    pub fn scalar_part(&self) -> T {
        self.data[3]
    }

    /// Vector4として取得
    pub fn to_vector4(&self) -> Vector4<T> {
        Vector4::new(self.data[0], self.data[1], self.data[2], self.data[3])
    }

    // === 演算 ===

    /// ノルム（大きさ）を計算
    pub fn norm(&self) -> T {
        (self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]
            + self.data[3] * self.data[3])
            .sqrt()
    }

    /// ノルムの2乗を計算
    pub fn norm_squared(&self) -> T {
        self.data[0] * self.data[0]
            + self.data[1] * self.data[1]
            + self.data[2] * self.data[2]
            + self.data[3] * self.data[3]
    }

    /// 正規化（単位クォータニオン化）
    pub fn normalize(&self) -> Result<Self, String> {
        let norm = self.norm();
        if norm.is_zero() {
            return Err("Cannot normalize zero quaternion".to_string());
        }
        Ok(Self::new(
            self.data[0] / norm,
            self.data[1] / norm,
            self.data[2] / norm,
            self.data[3] / norm,
        ))
    }

    /// 共役クォータニオン
    pub fn conjugate(&self) -> Self {
        Self::new(-self.data[0], -self.data[1], -self.data[2], self.data[3])
    }

    /// 逆クォータニオン
    pub fn inverse(&self) -> Result<Self, String> {
        let norm_sq = self.norm_squared();
        if norm_sq.is_zero() {
            return Err("Cannot invert zero quaternion".to_string());
        }
        let conjugate = self.conjugate();
        Ok(Self::new(
            conjugate.data[0] / norm_sq,
            conjugate.data[1] / norm_sq,
            conjugate.data[2] / norm_sq,
            conjugate.data[3] / norm_sq,
        ))
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> T {
        self.data[0] * other.data[0]
            + self.data[1] * other.data[1]
            + self.data[2] * other.data[2]
            + self.data[3] * other.data[3]
    }

    // === 回転操作 ===

    /// ベクトルを回転
    pub fn rotate_vector(&self, v: &Vector3<T>) -> Vector3<T> {
        // q * (0, v) * q^(-1) = (0, rotated_v)
        let qv = Quaternion::new(v.x(), v.y(), v.z(), T::ZERO);
        let result = *self * qv * self.conjugate();
        result.vector_part()
    }

    /// クォータニオンを軸角表現に変換
    pub fn to_axis_angle(&self) -> Result<(Vector3<T>, T), String> {
        let normalized = self.normalize()?;
        let w = normalized.w().clamp(-T::ONE, T::ONE);
        let angle = T::from_f64(2.0) * w.acos();

        let sin_half_angle = (T::ONE - w * w).sqrt();

        if sin_half_angle < T::EPSILON {
            // 回転角が0に近い場合
            return Ok((Vector3::new(T::ONE, T::ZERO, T::ZERO), T::ZERO));
        }

        let axis = Vector3::new(
            normalized.x() / sin_half_angle,
            normalized.y() / sin_half_angle,
            normalized.z() / sin_half_angle,
        );

        Ok((axis, angle))
    }

    /// オイラー角に変換（XYZ順）
    pub fn to_euler_angles(&self) -> (T, T, T) {
        let normalized = self.normalize().unwrap_or(*self);

        let x = normalized.x();
        let y = normalized.y();
        let z = normalized.z();
        let w = normalized.w();

        // Roll (x-axis rotation)
        let sin_r_cp = T::from_f64(2.0) * (w * x + y * z);
        let cos_r_cp = T::ONE - T::from_f64(2.0) * (x * x + y * y);
        let roll = sin_r_cp.atan2(cos_r_cp);

        // Pitch (y-axis rotation)
        let sin_p = T::from_f64(2.0) * (w * y - z * x);
        let pitch = if sin_p.abs() >= T::ONE {
            if sin_p >= T::ZERO {
                T::PI / T::from_f64(2.0)
            } else {
                -T::PI / T::from_f64(2.0)
            }
        } else {
            sin_p.asin()
        };

        // Yaw (z-axis rotation)
        let sin_y_cp = T::from_f64(2.0) * (w * z + x * y);
        let cos_y_cp = T::ONE - T::from_f64(2.0) * (y * y + z * z);
        let yaw = sin_y_cp.atan2(cos_y_cp);

        (pitch, yaw, roll)
    }

    // === 補間 ===

    /// 線形補間（LERP）
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let result = *self * (T::ONE - t) + *other * t;
        result.normalize().unwrap_or(result)
    }

    /// 球面線形補間（SLERP）
    pub fn slerp(&self, other: &Self, t: T) -> Result<Self, String> {
        let mut dot = self.dot(other);

        // 最短経路を選択
        let other = if dot < T::ZERO {
            dot = -dot;
            -*other
        } else {
            *other
        };

        // 角度が小さい場合は線形補間
        if dot > T::from_f64(thresholds::SLERP_THRESHOLD) {
            return Ok(self.lerp(&other, t));
        }

        let theta = dot.clamp(-T::ONE, T::ONE).acos();
        let sin_theta = theta.sin();

        if sin_theta.abs() < T::EPSILON {
            return Ok(*self);
        }

        let s0 = ((T::ONE - t) * theta).sin() / sin_theta;
        let s1 = (t * theta).sin() / sin_theta;

        Ok(*self * s0 + other * s1)
    }

    /// 正規化された球面線形補間（NLERP）
    pub fn nlerp(&self, other: &Self, t: T) -> Result<Self, String> {
        self.lerp(other, t).normalize()
    }

    // === ユーティリティ ===

    /// 単位クォータニオンかどうか判定
    pub fn is_unit(&self) -> bool {
        (self.norm() - T::ONE).abs() < T::EPSILON
    }

    /// ゼロクォータニオンかどうか判定
    pub fn is_zero(&self) -> bool {
        self.data[0].abs() < T::EPSILON
            && self.data[1].abs() < T::EPSILON
            && self.data[2].abs() < T::EPSILON
            && self.data[3].abs() < T::EPSILON
    }

    /// 回転角度を取得（ラジアン）
    pub fn angle(&self) -> T {
        T::from_f64(2.0) * self.w().abs().clamp(T::ZERO, T::ONE).acos()
    }
}

// === 演算子オーバーロード ===

impl<T: Scalar> Add for Quaternion<T> {
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

impl<T: Scalar> Sub for Quaternion<T> {
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

impl<T: Scalar> Mul<T> for Quaternion<T> {
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

impl<T: Scalar> Mul for Quaternion<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        // クォータニオンの積 (Hamilton product)
        Self::new(
            self.data[3] * other.data[0]
                + self.data[0] * other.data[3]
                + self.data[1] * other.data[2]
                - self.data[2] * other.data[1],
            self.data[3] * other.data[1] - self.data[0] * other.data[2]
                + self.data[1] * other.data[3]
                + self.data[2] * other.data[0],
            self.data[3] * other.data[2] + self.data[0] * other.data[1]
                - self.data[1] * other.data[0]
                + self.data[2] * other.data[3],
            self.data[3] * other.data[3]
                - self.data[0] * other.data[0]
                - self.data[1] * other.data[1]
                - self.data[2] * other.data[2],
        )
    }
}

impl<T: Scalar> Neg for Quaternion<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.data[0], -self.data[1], -self.data[2], -self.data[3])
    }
}

// === 添え字演算子 ===

impl<T: Scalar> Index<usize> for Quaternion<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

impl<T: Scalar> IndexMut<usize> for Quaternion<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.data[index]
    }
}

// === 配列変換 ===

impl<T: Scalar> From<[T; 4]> for Quaternion<T> {
    #[inline]
    fn from(data: [T; 4]) -> Self {
        Self { data }
    }
}

/// 型エイリアス
pub type Quaternionf = Quaternion<f32>;
pub type Quaterniond = Quaternion<f64>;
