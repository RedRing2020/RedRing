//! ベクトル（Vector）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Vector3D の実装

use crate::{BBox3D, Point3D};
use geo_foundation::{abstract_types::foundation::core_foundation::*, Scalar};

/// 3次元ベクトル
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Vector3D<T> {
    /// 新しいベクトルを作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// ゼロベクトルを取得
    pub fn zero() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// 単位ベクトル（軸方向）
    pub fn unit_x() -> Self {
        Self::new(T::ONE, T::ZERO, T::ZERO)
    }

    pub fn unit_y() -> Self {
        Self::new(T::ZERO, T::ONE, T::ZERO)
    }

    pub fn unit_z() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ONE)
    }

    /// X成分を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y成分を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// Z成分を取得
    pub fn z(&self) -> T {
        self.z
    }

    /// 成分を配列として取得
    pub fn components(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    /// ベクトルの長さ（ノルム）を計算
    pub fn length(&self) -> T {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// ベクトルの長さの2乗を計算（sqrt計算を避ける）
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// ベクトルを正規化（単位ベクトル化）
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len > T::EPSILON {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        } else {
            None // ゼロベクトルは正規化できない
        }
    }

    /// ベクトルの反転
    pub fn negate(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }

    /// 内積計算
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積計算
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// 2点間のベクトルを作成
    pub fn from_points(from: &Point3D<T>, to: &Point3D<T>) -> Self {
        Self::new(to.x() - from.x(), to.y() - from.y(), to.z() - from.z())
    }

    /// ベクトルを点として解釈（原点からの位置ベクトル）
    pub fn to_point(&self) -> Point3D<T> {
        Point3D::new(self.x, self.y, self.z)
    }

    /// ゼロベクトルかどうかを判定
    pub fn is_zero(&self) -> bool {
        self.length() <= T::EPSILON
    }

    /// 他のベクトルと平行かどうかを判定
    pub fn is_parallel(&self, other: &Self) -> bool {
        let cross = self.cross(other);
        cross.length() <= T::TOLERANCE
    }

    /// 他のベクトルと垂直かどうかを判定
    pub fn is_perpendicular(&self, other: &Self) -> bool {
        self.dot(other).abs() <= T::TOLERANCE
    }
}

// === 演算子オーバーロード ===

impl<T: Scalar> std::ops::Mul<T> for Vector3D<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl<T: Scalar> std::ops::Add for Vector3D<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Scalar> std::ops::Sub for Vector3D<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// === foundation トレイト実装 ===

impl<T: Scalar> CoreFoundation<T> for Vector3D<T> {
    type Point = Point3D<T>;
    type Vector = Self;
    type BBox = BBox3D<T>;

    /// ベクトルの境界ボックス（原点と終点を含む）
    fn bounding_box(&self) -> Self::BBox {
        let origin = Point3D::<T>::origin();
        let end_point = self.to_point();

        let min_x = origin.x().min(end_point.x());
        let max_x = origin.x().max(end_point.x());
        let min_y = origin.y().min(end_point.y());
        let max_y = origin.y().max(end_point.y());
        let min_z = origin.z().min(end_point.z());
        let max_z = origin.z().max(end_point.z());

        BBox3D::new(
            Point3D::new(min_x, min_y, min_z),
            Point3D::new(max_x, max_y, max_z),
        )
    }
}

impl<T: Scalar> BasicMetrics<T> for Vector3D<T> {
    /// ベクトルの長さ
    fn length(&self) -> Option<T> {
        Some(self.length())
    }
}

impl<T: Scalar> BasicDirectional<T> for Vector3D<T> {
    type Direction = Self;

    /// 方向（正規化されたベクトル）
    fn direction(&self) -> Self::Direction {
        self.normalize().unwrap_or_else(|| Self::unit_x()) // デフォルトは X 軸方向
    }

    /// 方向を反転
    fn reverse_direction(&self) -> Self {
        self.negate()
    }
}

// 基本機能のみに集中 - 複雑な変換は将来のextensionトレイトで実装予定
//
// 削除済みの複雑な機能：
// - rotate_around_axis (行列変換として別途実装予定)
// - 複合変換操作
// - 高度な幾何計算
