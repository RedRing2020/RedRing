use std::ops::{Add, Sub};
use super::vector::Vector;
use crate::geometry::geometry3d;
use crate::geometry_trait::point_ops::PointOps;
use geo_core::{Point3D as GeoPoint3D, ToleranceContext, TolerantEq};

/// ハイブリッド統合Point: geo_coreの数値堅牢性 + model既存API
#[derive(Debug, Clone)]
pub struct Point {
    inner: GeoPoint3D,  // geo_coreの数値基盤を活用
}

impl Point {
    /// コンストラクタ：明示的に座標を指定
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: GeoPoint3D::from_f64(x, y, z),
        }
    }

    /// ゲッター：各成分を取得
    pub fn x(&self) -> f64 {
        self.inner.x().value()
    }

    pub fn y(&self) -> f64 {
        self.inner.y().value()
    }

    pub fn z(&self) -> f64 {
        self.inner.z().value()
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> f64 {
        self.inner.distance_to(&other.inner).value()
    }

    /// 自身をベクトルとして取得（原点からの位置ベクトル）
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x(), self.y(), self.z())
    }

    /// ベクトルで平行移動
    pub fn translate(&self, v: &Vector) -> Self {
        Self::new(self.x() + v.x(), self.y() + v.y(), self.z() + v.z())
    }
    
    /// トレラント比較（geo_coreの数値堅牢性を活用）
    pub fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        self.inner.tolerant_eq(&other.inner, context)
    }
    
    /// 原点からの点を作成（便利メソッド）
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
    
    /// geo_coreへの内部アクセス（高度な数値演算用）
    pub fn as_geo_core(&self) -> &GeoPoint3D {
        &self.inner
    }
    
    /// geo_coreからの変換
    pub fn from_geo_core(inner: GeoPoint3D) -> Self {
        Self { inner }
    }
}

impl PointOps for geometry3d::point::Point {
    fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn add(&self, other: &Self) -> Self {
        Self::new(self.x() + other.x(), self.y() + other.y(), self.z() + other.z())
    }
    
    fn sub(&self, other: &Self) -> Self {
        Self::new(self.x() - other.x(), self.y() - other.y(), self.z() - other.z())
    }
    
    fn mul(&self, scalar: f64) -> Self {
        Self::new(self.x() * scalar, self.y() * scalar, self.z() * scalar)
    }

    fn div(&self, scalar: f64) -> Self {
        Self::new(self.x() / scalar, self.y() / scalar, self.z() / scalar)
    }

    fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        Self::new(
            self.x() + other.x() * scale,
            self.y() + other.y() * scale,
            self.z() + other.z() * scale
        )
    }
}

impl Add<Vector> for Point {
    type Output = Point;
    fn add(self, rhs: Vector) -> Point {
        Point::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl Sub for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Vector {
        Vector::new(self.x() - rhs.x(), self.y() - rhs.y(), self.z() - rhs.z())
    }
}

// PartialEq実装（既存コードとの互換性のため）
// 注意: geo_core::Point3DはCopyを実装していないため、Copyは削除

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        // デフォルト許容誤差での比較
        let default_ctx = ToleranceContext::default();
        self.inner.tolerant_eq(&other.inner, &default_ctx)
    }
}
