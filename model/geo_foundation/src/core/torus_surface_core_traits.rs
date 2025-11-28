//! TorusSurface Core Traits - トーラスサーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_torus_surface）
//! - Properties: 6メソッド（center, major_radius, minor_radius, axis, ref_direction, tube_diameter）
//! - Measure: 4メソッド（surface_area, point_at_uv, normal_at, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - TorusSurface生成機能（Phase 1: 最小限）
// ============================================================================

/// TorusSurface3D生成のためのConstructorトレイト
pub trait TorusSurface3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式でトーラスサーフェスを作成
    ///
    /// # Arguments
    /// * `center` - トーラスの中心点（x, y, z）
    /// * `axis` - トーラスの軸方向ベクトル（Z軸、回転軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `major_radius` - 主半径（中心から管の中心までの距離、正の値）
    /// * `minor_radius` - 副半径（管の半径、正の値）
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        major_radius: T,
        minor_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準のトーラスサーフェスを作成（簡易コンストラクタ）
    fn new_standard(center: (T, T, T), major_radius: T, minor_radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位トーラスサーフェス（主半径2、副半径1）
    fn unit_torus_surface() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - TorusSurface基本情報取得（Phase 1: 最小限）
// ============================================================================

/// TorusSurface3D基本プロパティ取得トレイト
pub trait TorusSurface3DProperties<T: Scalar> {
    /// トーラスの中心点取得
    fn center(&self) -> (T, T, T);

    /// 主半径取得（中心から管の中心までの距離）
    fn major_radius(&self) -> T;

    /// 副半径取得（管の半径）
    fn minor_radius(&self) -> T;

    /// 軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 管の直径取得
    fn tube_diameter(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - TorusSurface測定機能（Phase 1: 最小限）
// ============================================================================

/// TorusSurface3D測定機能トレイト
pub trait TorusSurface3DMeasure<T: Scalar> {
    /// トーラスサーフェスの表面積を計算
    ///
    /// 表面積 = 4π² × R × r
    fn surface_area(&self) -> T;

    /// パラメータ座標(u, v)から表面上の点を計算
    ///
    /// u ∈ [0, 2π]: 主円方向の角度
    /// v ∈ [0, 2π]: 管の円周方向の角度
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> (T, T, T);

    /// 点とサーフェスとの最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// TorusSurface3DのCore機能を統合するトレイト
pub trait TorusSurface3DCore<T: Scalar>:
    TorusSurface3DConstructor<T> + TorusSurface3DProperties<T> + TorusSurface3DMeasure<T>
{
}
