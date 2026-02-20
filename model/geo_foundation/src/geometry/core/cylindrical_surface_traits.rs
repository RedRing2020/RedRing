//! CylindricalSurface Core Traits - 円柱サーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_cylinder_surface）
//! - Properties: 6メソッド（center, radius, height, axis, ref_direction, diameter）
//! - Measure: 4メソッド（surface_area, point_at_uv, normal_at, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - CylindricalSurface生成機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSurface3D生成のためのConstructorトレイト
pub trait CylindricalSurface3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式で円柱サーフェスを作成
    ///
    /// # Arguments
    /// * `center` - 円柱底面の中心点（x, y, z）
    /// * `axis` - 円柱の軸方向ベクトル（Z軸、高さ方向）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `radius` - 底面の半径（正の値）
    /// * `height` - 円柱の高さ（正の値）
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の円柱サーフェスを作成（簡易コンストラクタ）
    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位円柱サーフェス（半径1、高さ2）
    fn unit_cylinder_surface() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - CylindricalSurface基本情報取得（Phase 1: 最小限）
// ============================================================================

/// CylindricalSurface3D基本プロパティ取得トレイト
pub trait CylindricalSurface3DProperties<T: Scalar> {
    /// 円柱底面の中心点取得
    fn center(&self) -> (T, T, T);

    /// 円柱底面の半径取得
    fn radius(&self) -> T;

    /// 円柱の高さ取得
    fn height(&self) -> T;

    /// 軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 円柱底面の直径取得
    fn diameter(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - CylindricalSurface測定機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSurface3D測定機能トレイト
pub trait CylindricalSurface3DMeasure<T: Scalar> {
    /// 円柱サーフェスの表面積を計算
    ///
    /// 表面積 = 2π × r × h（側面のみ）
    fn surface_area(&self) -> T;

    /// パラメータ座標(u, v)から表面上の点を計算
    ///
    /// u ∈ [0, 2π]: 方位角
    /// v ∈ [0, h]: 高さ方向
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> (T, T, T);

    /// 点とサーフェスとの最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// CylindricalSurface3DのCore機能を統合するトレイト
pub trait CylindricalSurface3DCore<T: Scalar>:
    CylindricalSurface3DConstructor<T>
    + CylindricalSurface3DProperties<T>
    + CylindricalSurface3DMeasure<T>
{
}
