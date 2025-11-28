//! SphericalSurface Core Traits - 球サーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_sphere_surface）
//! - Properties: 5メソッド（center, radius, axis, ref_direction, diameter）
//! - Measure: 4メソッド（surface_area, point_at_uv, normal_at, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - SphericalSurface生成機能（Phase 1: 最小限）
// ============================================================================

/// SphericalSurface3D生成のためのConstructorトレイト
pub trait SphericalSurface3DConstructor<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本コンストラクタ（3メソッド）
    // ========================================================================

    /// STEP準拠のAXIS2_PLACEMENT_3D形式で球サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 球の中心点（x, y, z）
    /// * `axis` - 参照軸ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `radius` - 半径（正の値）
    fn new(center: (T, T, T), axis: (T, T, T), ref_direction: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の球サーフェスを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位球サーフェス（半径1）
    fn unit_sphere_surface() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - SphericalSurface基本情報取得（Phase 1: 最小限）
// ============================================================================

/// SphericalSurface3D基本プロパティ取得トレイト
pub trait SphericalSurface3DProperties<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本プロパティ（5メソッド）
    // ========================================================================

    /// 球の中心点取得
    fn center(&self) -> (T, T, T);

    /// 球の半径取得
    fn radius(&self) -> T;

    /// 参照軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 球の直径取得
    fn diameter(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - SphericalSurface測定機能（Phase 1: 最小限）
// ============================================================================

/// SphericalSurface3D測定機能トレイト
pub trait SphericalSurface3DMeasure<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本測定（4メソッド）
    // ========================================================================

    /// 球サーフェスの表面積を計算
    ///
    /// 表面積 = 4π × r²
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

/// SphericalSurface3DのCore機能を統合するトレイト
///
/// Constructor/Properties/Measureの全機能を統合
pub trait SphericalSurface3DCore<T: Scalar>:
    SphericalSurface3DConstructor<T> + SphericalSurface3DProperties<T> + SphericalSurface3DMeasure<T>
{
}
