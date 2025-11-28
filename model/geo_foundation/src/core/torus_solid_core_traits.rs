//! TorusSolid Core Traits - トーラスソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_torus）
//! - Properties: 6メソッド（center, major_radius, minor_radius, axis, ref_direction, tube_diameter）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - TorusSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// TorusSolid3D生成のためのConstructorトレイト
pub trait TorusSolid3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式でトーラスソリッドを作成
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

    /// Z軸標準のトーラスソリッドを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), major_radius: T, minor_radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位トーラス（主半径2、副半径1）
    fn unit_torus() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - TorusSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// TorusSolid3D基本プロパティ取得トレイト
pub trait TorusSolid3DProperties<T: Scalar> {
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
// 3. Measure Traits - TorusSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// TorusSolid3D測定機能トレイト
pub trait TorusSolid3DMeasure<T: Scalar> {
    /// トーラスの体積を計算
    ///
    /// 体積 = 2π² × R × r²
    /// （R: 主半径, r: 副半径）
    fn volume(&self) -> T;

    /// トーラスの表面積を計算
    ///
    /// 表面積 = 4π² × R × r
    fn surface_area(&self) -> T;

    /// 点がトーラス内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点とトーラスとの最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// TorusSolid3DのCore機能を統合するトレイト
pub trait TorusSolid3DCore<T: Scalar>:
    TorusSolid3DConstructor<T> + TorusSolid3DProperties<T> + TorusSolid3DMeasure<T>
{
}
