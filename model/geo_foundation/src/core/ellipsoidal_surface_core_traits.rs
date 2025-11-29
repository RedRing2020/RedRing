//! EllipsoidalSurface Core Traits - 楕円体サーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1: 最小限のメソッド
//! - Constructor: 3メソッド（new, new_standard, unit_ellipsoid）
//! - Properties: 7メソッド（center, semi_axis_a/b/c, axis, ref_direction）
//! - Measure: 4メソッド（approximate_surface_area, point_at_uv, normal_at, distance_to_point）
//!
//! ## Phase 2: 標準機能追加
//! - Constructor: +3メソッド（from_semi_axes, from_bounding_box, oblate_spheroid）
//! - Properties: +3メソッド（eccentricity, is_sphere, is_oblate）
//! - Measure: +4メソッド（point_at_spherical, bounding_box, volume, surface_area_knud_thomsen）

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

    // Phase 2: 追加コンストラクタ（3メソッド）

    /// 3軸半径指定で楕円体を作成（簡易版、標準軸配置）
    fn from_semi_axes(center: (T, T, T), a: T, b: T, c: T) -> Option<Self>
    where
        Self: Sized;

    /// 境界ボックスに内接する楕円体を作成
    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 扁平回転楕円体を作成（極半径 < 赤道半径）
    fn oblate_spheroid(
        center: (T, T, T),
        equatorial_radius: T,
        polar_radius: T,
    ) -> Option<Self>
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

    // Phase 2: 追加プロパティ（3メソッド）

    /// 離心率を取得
    fn eccentricity(&self) -> T;

    /// 球かどうか判定（a = b = c）
    fn is_sphere(&self) -> bool;

    /// 扁平形状かどうか判定（極半径 < 赤道半径）
    fn is_oblate(&self) -> bool;
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

    // Phase 2: 追加測定（4メソッド）

    /// 球面座標系での点を取得
    ///
    /// theta ∈ [0, 2π]: 方位角, phi ∈ [0, π]: 仰角
    fn point_at_spherical(&self, theta: T, phi: T) -> (T, T, T);

    /// 楕円体の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));

    /// 楕円体の体積を計算（(4/3)π × a × b × c）
    fn volume(&self) -> T;

    /// Knud Thomsen's formula による近似表面積
    fn surface_area_knud_thomsen(&self) -> T;
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
