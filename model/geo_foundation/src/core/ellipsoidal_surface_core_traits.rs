//! EllipsoidalSurface Core Traits - 楕円体サーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_ellipsoid_surface）
//! - Properties: 6メソッド（center, semi_axis_a, semi_axis_b, semi_axis_c, axis, ref_direction）
//! - Measure: 4メソッド（surface_area, point_at_uv, normal_at, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - EllipsoidalSurface生成機能（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSurface3D生成のためのConstructorトレイト
pub trait EllipsoidalSurface3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式で楕円体サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 楕円体の中心点（x, y, z）
    /// * `axis` - 楕円体の主軸方向ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `semi_axis_a` - X軸方向の半径（正の値）
    /// * `semi_axis_b` - Y軸方向の半径（正の値）
    /// * `semi_axis_c` - Z軸方向の半径（正の値）
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        semi_axis_a: T,
        semi_axis_b: T,
        semi_axis_c: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の楕円体サーフェスを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(
        center: (T, T, T),
        semi_axis_a: T,
        semi_axis_b: T,
        semi_axis_c: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位球サーフェス（a=b=c=1）
    fn unit_ellipsoid_surface() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - EllipsoidalSurface基本情報取得（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSurface3D基本プロパティ取得トレイト
pub trait EllipsoidalSurface3DProperties<T: Scalar> {
    /// 楕円体の中心点取得
    fn center(&self) -> (T, T, T);

    /// X軸方向の半径取得
    fn semi_axis_a(&self) -> T;

    /// Y軸方向の半径取得
    fn semi_axis_b(&self) -> T;

    /// Z軸方向の半径取得
    fn semi_axis_c(&self) -> T;

    /// 主軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);
}

// ============================================================================
// 3. Measure Traits - EllipsoidalSurface測定機能（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSurface3D測定機能トレイト
pub trait EllipsoidalSurface3DMeasure<T: Scalar> {
    /// 楕円体サーフェスの表面積を計算
    ///
    /// 厳密解は楕円積分で表現されるため、近似計算を使用
    /// Knud Thomsen's formula: S ≈ 4π × ((ab)^p + (ac)^p + (bc)^p) / 3)^(1/p)
    /// where p ≈ 1.6075
    fn surface_area(&self) -> T;

    /// パラメータ座標(u, v)から表面上の点を計算
    ///
    /// u ∈ [0, 2π]: 方位角（経度）
    /// v ∈ [-π/2, π/2]: 仰角（緯度）
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> (T, T, T);

    /// 点とサーフェスとの最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// EllipsoidalSurface3DのCore機能を統合するトレイト
pub trait EllipsoidalSurface3DCore<T: Scalar>:
    EllipsoidalSurface3DConstructor<T>
    + EllipsoidalSurface3DProperties<T>
    + EllipsoidalSurface3DMeasure<T>
{
}
