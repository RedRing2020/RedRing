//! CylindricalSolid Core Traits - 円柱ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_standard, unit_cylinder）
//! - Properties: 6メソッド（center, radius, height, axis, ref_direction, diameter）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_point）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - CylindricalSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D生成のためのConstructorトレイト
pub trait CylindricalSolid3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式で円柱ソリッドを作成
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

    /// Z軸標準の円柱ソリッドを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位円柱（半径1、高さ2）
    fn unit_cylinder() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - CylindricalSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D基本プロパティ取得トレイト
pub trait CylindricalSolid3DProperties<T: Scalar> {
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
// 3. Measure Traits - CylindricalSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D測定機能トレイト
pub trait CylindricalSolid3DMeasure<T: Scalar> {
    /// 円柱の体積を計算
    ///
    /// 体積 = π × r² × h
    fn volume(&self) -> T;

    /// 円柱の表面積を計算
    ///
    /// 表面積 = 2π × r × (r + h)
    fn surface_area(&self) -> T;

    /// 点が円柱内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点と円柱との最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// CylindricalSolid3DのCore機能を統合するトレイト
pub trait CylindricalSolid3DCore<T: Scalar>:
    CylindricalSolid3DConstructor<T> + CylindricalSolid3DProperties<T> + CylindricalSolid3DMeasure<T>
{
}
