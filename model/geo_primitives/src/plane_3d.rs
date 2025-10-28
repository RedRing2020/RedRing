//! Plane3D Core 実装
//!
//! Foundation統一システムに基づくPlane3Dの必須機能のみ
//! 平面は点と法線ベクトルで定義されます

use crate::{Point3D, Vector3D};
use geo_foundation::Scalar;

/// 3次元空間の平面
///
/// 平面は基準点 (point) と法線ベクトル (normal) で定義されます。
/// 平面上の任意の点 P について、(P - point) · normal = 0 が成り立ちます。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane3D<T: Scalar> {
    /// 平面上の基準点
    point: Point3D<T>,
    /// 平面の法線ベクトル（正規化されている必要があります）
    normal: Vector3D<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Plane3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 点と法線ベクトルから平面を作成
    ///
    /// # Arguments
    /// * `point` - 平面上の基準点
    /// * `normal` - 平面の法線ベクトル（正規化される）
    ///
    /// # Returns
    /// 新しい平面、法線ベクトルがゼロベクトルの場合は None
    pub fn from_point_and_normal(point: Point3D<T>, normal: Vector3D<T>) -> Option<Self> {
        let length = normal.length();
        if length == T::ZERO {
            return None;
        }

        let normalized_normal = normal / length;
        Some(Self {
            point,
            normal: normalized_normal,
        })
    }

    /// 3つの点から平面を作成
    ///
    /// # Arguments
    /// * `p1`, `p2`, `p3` - 平面上の3点（一直線上にない）
    ///
    /// # Returns
    /// 新しい平面、3点が一直線上にある場合は None
    pub fn from_three_points(p1: Point3D<T>, p2: Point3D<T>, p3: Point3D<T>) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2);

        Self::from_point_and_normal(p1, normal)
    }

    /// XY平面を作成（z = constant）
    pub fn xy_plane(z: T) -> Self {
        Self {
            point: Point3D::new(T::ZERO, T::ZERO, z),
            normal: Vector3D::unit_z(),
        }
    }

    /// XZ平面を作成（y = constant）
    pub fn xz_plane(y: T) -> Self {
        Self {
            point: Point3D::new(T::ZERO, y, T::ZERO),
            normal: Vector3D::unit_y(),
        }
    }

    /// YZ平面を作成（x = constant）
    pub fn yz_plane(x: T) -> Self {
        Self {
            point: Point3D::new(x, T::ZERO, T::ZERO),
            normal: Vector3D::unit_x(),
        }
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 平面上の基準点を取得
    pub fn point(&self) -> Point3D<T> {
        self.point
    }

    /// 平面の法線ベクトルを取得
    pub fn normal(&self) -> Vector3D<T> {
        self.normal
    }

    // ========================================================================
    // Core Geometric Operations
    // ========================================================================

    /// 点が平面上にあるかチェック
    ///
    /// # Arguments
    /// * `point` - チェックする点
    /// * `tolerance` - 許容誤差
    ///
    /// # Returns
    /// 点が平面上にある場合 true
    pub fn contains_point(&self, point: Point3D<T>, tolerance: T) -> bool {
        let distance = self.distance_to_point(point).abs();
        distance <= tolerance
    }

    /// 点から平面までの符号付き距離
    ///
    /// # Arguments
    /// * `point` - 距離を求める点
    ///
    /// # Returns
    /// 符号付き距離（法線ベクトル方向が正）
    pub fn distance_to_point(&self, point: Point3D<T>) -> T {
        let to_point = point - self.point;
        to_point.dot(&self.normal)
    }

    /// 点を平面に投影
    ///
    /// # Arguments
    /// * `point` - 投影する点
    ///
    /// # Returns
    /// 平面上の投影点
    pub fn project_point(&self, point: Point3D<T>) -> Point3D<T> {
        let distance = self.distance_to_point(point);
        point - self.normal * distance
    }

    /// 平面の方程式係数を取得
    ///
    /// # Returns
    /// (a, b, c, d) where ax + by + cz + d = 0
    pub fn equation_coefficients(&self) -> (T, T, T, T) {
        let a = self.normal.x();
        let b = self.normal.y();
        let c = self.normal.z();
        let d = -(a * self.point.x() + b * self.point.y() + c * self.point.z());
        (a, b, c, d)
    }

    // ========================================================================
    // Core Validation Methods
    // ========================================================================

    /// 平面が有効かチェック
    pub fn is_valid(&self) -> bool {
        let normal_length = self.normal.length();
        (normal_length - T::ONE).abs() < T::from_f64(1e-10)
    }
}

// ============================================================================
// Core Trait Implementations
// ============================================================================

impl<T: Scalar> Default for Plane3D<T> {
    /// デフォルトはXY平面（z = 0）
    fn default() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

// ============================================================================
// Constants (注: ジェネリック型では const は制限があるため、メソッドで提供)
// ============================================================================

impl<T: Scalar> Plane3D<T> {
    /// XY平面（z = 0）の参照
    pub fn xy() -> Self {
        Self::xy_plane(T::ZERO)
    }
}

// ============================================================================
// Display Implementation
// ============================================================================

impl<T: Scalar + std::fmt::Display> std::fmt::Display for Plane3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plane3D(point: ({}, {}, {}), normal: ({}, {}, {}))",
            self.point.x(),
            self.point.y(),
            self.point.z(),
            self.normal.x(),
            self.normal.y(),
            self.normal.z()
        )
    }
}
