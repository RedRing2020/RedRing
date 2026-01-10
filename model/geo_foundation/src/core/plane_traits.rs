//! Plane3D Core Traits - Plane形状のCore機能統合
//!
//! Foundation Pattern Phase 1+2 実装
//! 3-5-4 パターン: Constructor(3+3) + Properties(5+3) + Measure(4+4)
//!
//! 作成日: 2025年11月28日

use crate::Scalar;

// ============================================================================
// Plane3D Core Traits
// ============================================================================

/// Plane3D Constructor トレイト（3+3メソッド）
pub trait Plane3DConstructor<T: Scalar>: Sized {
    // ========== Phase 1 実装 ==========
    /// 原点と軸から平面を作成（STEP AXIS2_PLACEMENT_3D形式）
    fn from_origin_and_axes(
        origin: (T, T, T),
        normal: (T, T, T),
        u_direction: (T, T, T),
    ) -> Option<Self>;

    /// 3点から平面を作成
    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self>;

    /// XY平面を作成（Z=0）
    fn xy_plane() -> Self;

    // ========== Phase 2 実装 ==========
    /// 点と法線から平面を作成
    fn from_point_and_normal(point: (T, T, T), normal: (T, T, T)) -> Option<Self>;

    /// XZ平面を作成（Y=0）
    fn xz_plane() -> Self;

    /// YZ平面を作成（X=0）
    fn yz_plane() -> Self;
}

/// Plane3D Properties トレイト（5+3メソッド）
pub trait Plane3DProperties<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 原点座標を取得
    fn origin(&self) -> (T, T, T);

    /// 法線（Z軸）方向を取得
    fn normal(&self) -> (T, T, T);

    /// U軸（X軸）方向を取得
    fn u_axis(&self) -> (T, T, T);

    /// V軸（Y軸）方向を取得
    fn v_axis(&self) -> (T, T, T);

    /// 形状の次元数（2: 2次元多様体）
    fn dimension(&self) -> u32;

    // ========== Phase 2 実装 ==========
    /// XY平面かどうか判定
    fn is_xy_plane(&self) -> bool;

    /// XZ平面かどうか判定
    fn is_xz_plane(&self) -> bool;

    /// YZ平面かどうか判定
    fn is_yz_plane(&self) -> bool;
}

/// Plane3D Measure トレイト（4+4メソッド）
pub trait Plane3DMeasure<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 点が平面上にあるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点から平面への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 点を平面に投影
    fn project_point(&self, point: (T, T, T)) -> (T, T, T);

    /// 平面の方程式係数を取得（Ax + By + Cz + D = 0）
    fn equation_coefficients(&self) -> (T, T, T, T);

    // ========== Phase 2 実装 ==========
    /// 点の平面座標（UV座標）を取得
    fn point_to_uv(&self, point: (T, T, T)) -> (T, T);

    /// UV座標から3D点を取得
    fn uv_to_point(&self, u: T, v: T) -> (T, T, T);

    /// 点を平面に対して鏡面反射
    fn mirror_point(&self, point: (T, T, T)) -> (T, T, T);

    /// 他の平面との交線を計算（方向ベクトルと通過点を返す）
    #[allow(clippy::type_complexity)]
    fn intersection_with_plane(
        &self,
        other_origin: (T, T, T),
        other_normal: (T, T, T),
    ) -> Option<((T, T, T), (T, T, T))>;
}

/// Plane3D Core トレイト（統合インターフェース）
pub trait Plane3DCore<T: Scalar>:
    Plane3DConstructor<T> + Plane3DProperties<T> + Plane3DMeasure<T>
{
}
