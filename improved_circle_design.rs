//! 改善されたCircle トレイト設計案
//!
//! ジェネリックでf32/f64両対応、2D/3D統一

use crate::common::constants::{PI, TAU};

/// 数値型の制約を定義するトレイト
pub trait Scalar: Copy + Clone + PartialEq + PartialOrd + 
    std::ops::Add<Output = Self> + 
    std::ops::Sub<Output = Self> + 
    std::ops::Mul<Output = Self> + 
    std::ops::Div<Output = Self> +
    std::fmt::Debug + 
    'static
{
    /// π定数
    fn pi() -> Self;
    /// 2π定数
    fn tau() -> Self;
    /// ゼロ値
    fn zero() -> Self;
    /// 1値
    fn one() -> Self;
    /// 2値
    fn two() -> Self;
    /// 絶対値
    fn abs(self) -> Self;
    /// 平方根
    fn sqrt(self) -> Self;
    /// 正弦
    fn sin(self) -> Self;
    /// 余弦
    fn cos(self) -> Self;
    /// 有限値かチェック
    fn is_finite(self) -> bool;
}

impl Scalar for f32 {
    fn pi() -> Self { std::f32::consts::PI }
    fn tau() -> Self { std::f32::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn is_finite(self) -> bool { self.is_finite() }
}

impl Scalar for f64 {
    fn pi() -> Self { std::f64::consts::PI }
    fn tau() -> Self { std::f64::consts::TAU }
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
    fn two() -> Self { 2.0 }
    fn abs(self) -> Self { self.abs() }
    fn sqrt(self) -> Self { self.sqrt() }
    fn sin(self) -> Self { self.sin() }
    fn cos(self) -> Self { self.cos() }
    fn is_finite(self) -> bool { self.is_finite() }
}

/// 統一されたCircleトレイト（2D/3D対応、f32/f64対応）
pub trait Circle<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 境界ボックスの型
    type BoundingBox;

    /// 次元数を取得（2または3）
    fn dimension() -> usize;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;

    /// 円の面積を計算
    fn area(&self) -> T {
        T::pi() * self.radius() * self.radius()
    }

    /// 円の周長（円周）を計算
    fn circumference(&self) -> T {
        T::tau() * self.radius()
    }

    /// 円の直径を計算
    fn diameter(&self) -> T {
        T::two() * self.radius()
    }

    /// 指定された点が円の内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 指定された点が円周上にあるかを判定（許容誤差内）
    fn on_circumference(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 円周上の指定された角度（ラジアン）での点を取得
    fn point_at_angle(&self, angle: T) -> Self::Point;

    /// 円周上の指定された点での接線ベクトルを取得
    fn tangent_at_point(&self, point: &Self::Point) -> Option<Self::Vector>;

    /// 円周上の指定された角度での接線ベクトルを取得
    fn tangent_at_angle(&self, angle: T) -> Self::Vector;

    /// 円の境界ボックス（外接矩形/直方体）を取得
    fn bounding_box(&self) -> Self::BoundingBox;
}

/// 3D円専用の追加機能
pub trait Circle3D<T: Scalar>: Circle<T> {
    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 指定された点が円の平面上にあるかを判定
    fn point_on_plane(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 点を円の平面に投影
    fn project_point_to_plane(&self, point: &Self::Point) -> Self::Point;

    /// 円の平面への投影を2D円として取得
    fn to_2d(&self) -> impl Circle<T>;
}

/// 角度を表現する構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle<T: Scalar> {
    radians: T,
}

impl<T: Scalar> Angle<T> {
    /// ラジアンから角度を作成
    pub fn from_radians(radians: T) -> Self {
        Self { radians }
    }

    /// 度数から角度を作成
    pub fn from_degrees(degrees: T) -> Self {
        let radians = degrees * T::pi() / (T::one() + T::one() + T::one() + T::one() + T::one() + T::one()); // 180.0を表現
        Self { radians }
    }

    /// ラジアン値を取得
    pub fn radians(&self) -> T {
        self.radians
    }

    /// 度数値を取得
    pub fn degrees(&self) -> T {
        self.radians * (T::one() + T::one() + T::one() + T::one() + T::one() + T::one()) / T::pi() // 180.0を表現
    }

    /// 正規化（0-2π範囲）
    pub fn normalize(&self) -> Self {
        let tau = T::tau();
        let mut normalized = self.radians;
        while normalized < T::zero() {
            normalized = normalized + tau;
        }
        while normalized >= tau {
            normalized = normalized - tau;
        }
        Self { radians: normalized }
    }

    /// 角度の差を計算（最短経路）
    pub fn difference(&self, other: &Self) -> Self {
        let diff = other.radians - self.radians;
        let tau = T::tau();
        let half_tau = tau / T::two();
        
        if diff > half_tau {
            Self { radians: diff - tau }
        } else if diff < -half_tau {
            Self { radians: diff + tau }
        } else {
            Self { radians: diff }
        }
    }

    /// 角度の補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        let diff = self.difference(other);
        Self { radians: self.radians + diff.radians * t }
    }
}

/// Arcトレイト（Circleベース）
pub trait Arc<T: Scalar> {
    /// 基底の円の型
    type Circle: Circle<T>;

    /// 基底の円を取得
    fn circle(&self) -> &Self::Circle;

    /// 円弧の開始角度を取得
    fn start_angle(&self) -> Angle<T>;

    /// 円弧の終了角度を取得
    fn end_angle(&self) -> Angle<T>;

    /// 円弧の角度範囲を取得
    fn angle_span(&self) -> Angle<T> {
        self.start_angle().difference(&self.end_angle())
    }

    /// 円弧の弧長を計算
    fn arc_length(&self) -> T {
        self.circle().radius() * self.angle_span().radians().abs()
    }

    /// 円弧の開始点を取得
    fn start_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.start_angle().radians())
    }

    /// 円弧の終了点を取得
    fn end_point(&self) -> <Self::Circle as Circle<T>>::Point {
        self.circle().point_at_angle(self.end_angle().radians())
    }

    /// 円弧上の指定されたパラメータ（0.0〜1.0）での点を取得
    fn point_at_parameter(&self, t: T) -> <Self::Circle as Circle<T>>::Point {
        let interpolated_angle = self.start_angle().lerp(&self.end_angle(), t);
        self.circle().point_at_angle(interpolated_angle.radians())
    }

    /// 指定された点が円弧上にあるかを判定
    fn contains_point(&self, point: &<Self::Circle as Circle<T>>::Point, tolerance: T) -> bool;
}

// 便利な型エイリアス
pub type Angle32 = Angle<f32>;
pub type Angle64 = Angle<f64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_creation() {
        let angle = Angle64::from_degrees(90.0);
        let pi_2 = std::f64::consts::PI / 2.0;
        assert!((angle.radians() - pi_2).abs() < 1e-10);
    }

    #[test]
    fn test_angle_normalization() {
        let angle = Angle64::from_radians(3.0 * std::f64::consts::PI);
        let normalized = angle.normalize();
        assert!((normalized.radians() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_angle_difference() {
        let a1 = Angle64::from_degrees(10.0);
        let a2 = Angle64::from_degrees(350.0);
        let diff = a1.difference(&a2);
        // 最短経路は-20度であるべき
        assert!((diff.degrees() + 20.0).abs() < 1e-10);
    }
}