//! 点（Point）の新実装
//!
//! foundation.rs の基盤トレイトに基づく Point3D の実装

use crate::{BBox3D, Vector3D};
use geo_foundation::{abstract_types::geometry::foundation::*, Scalar};

/// 3次元空間の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D<T: Scalar> {
    x: T,
    y: T,
    z: T,
}

impl<T: Scalar> Point3D<T> {
    /// 新しい点を作成
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    /// 原点を取得
    pub fn origin() -> Self {
        Self::new(T::ZERO, T::ZERO, T::ZERO)
    }

    /// X座標を取得
    pub fn x(&self) -> T {
        self.x
    }

    /// Y座標を取得
    pub fn y(&self) -> T {
        self.y
    }

    /// Z座標を取得
    pub fn z(&self) -> T {
        self.z
    }

    /// 座標を配列として取得
    pub fn coords(&self) -> [T; 3] {
        [self.x, self.y, self.z]
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

// === foundation トレイト実装 ===

impl<T: Scalar> GeometryFoundation<T> for Point3D<T> {
    type Point = Self;
    type Vector = Vector3D<T>;
    type BBox = BBox3D<T>;

    /// 点の境界ボックス = 点自身
    fn bounding_box(&self) -> Self::BBox {
        BBox3D::from_point(*self)
    }
}

impl<T: Scalar> BasicContainment<T> for Point3D<T> {
    /// 点が自分自身と一致するかを判定
    fn contains_point(&self, point: &Self::Point) -> bool {
        self == point
    }

    /// 点が境界上（許容誤差内）にあるかを判定
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool {
        self.distance_to(point) <= tolerance
    }

    /// 点から点への距離
    fn distance_to_point(&self, point: &Self::Point) -> T {
        self.distance_to(point)
    }
}

// === 演算子の実装 ===

// Point - Point = Vector (2点間のベクトル)
impl<T: Scalar> std::ops::Sub for Point3D<T> {
    type Output = Vector3D<T>;

    fn sub(self, other: Self) -> Self::Output {
        Vector3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

// Point + Vector = Point (点をベクトル分移動)
impl<T: Scalar> std::ops::Add<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn add(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x + vector.x(),
            self.y + vector.y(),
            self.z + vector.z(),
        )
    }
}

// Point - Vector = Point (点をベクトル分逆移動)
impl<T: Scalar> std::ops::Sub<Vector3D<T>> for Point3D<T> {
    type Output = Point3D<T>;

    fn sub(self, vector: Vector3D<T>) -> Self::Output {
        Point3D::new(
            self.x - vector.x(),
            self.y - vector.y(),
            self.z - vector.z(),
        )
    }
}

// 基本機能のみに集中 - 複雑な変換は将来のextensionトレイトで実装予定
//
// 削除済みの複雑な機能：
// - rotate_around_axis (行列変換として別途実装予定)
// - 複合変換操作
// - 高度な幾何計算
