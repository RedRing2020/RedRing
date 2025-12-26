//! EllipsoidalSolid Core Traits - 楕円体ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1: 最小限のメソッド
//! - Constructor: 3メソッド（new, new_standard, unit_ellipsoid）
//! - Properties: 7メソッド（center, a_radius, b_radius, c_radius, axis, ref_direction, radii）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_surface）
//!
//! ## Phase 2: 標準機能追加
//! - Constructor: +3メソッド（from_radii, from_bounding_box, new_sphere）
//! - Properties: +3メソッド（is_sphere, is_unit_ellipsoid, is_centered_at_origin）
//! - Measure: +4メソッド（bounding_box, closest_point_on_surface, is_on_surface, is_degenerate）

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - EllipsoidalSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSolid3D生成のためのConstructorトレイト
pub trait EllipsoidalSolid3DConstructor<T: Scalar> {
    // Phase 1: 基本コンストラクタ（3メソッド）

    /// STEP準拠のAXIS2_PLACEMENT_3D形式で楕円体ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 楕円体の中心点（x, y, z）
    /// * `axis` - 参照軸ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `a_radius` - X軸方向の半径（正の値）
    /// * `b_radius` - Y軸方向の半径（正の値）
    /// * `c_radius` - Z軸方向の半径（正の値）
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        a_radius: T,
        b_radius: T,
        c_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の楕円体ソリッドを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), a_radius: T, b_radius: T, c_radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位楕円体ソリッド（全半径1）
    fn unit_ellipsoid() -> Self
    where
        Self: Sized;

    // Phase 2: 追加コンストラクタ（3メソッド）

    /// 3軸半径指定で楕円体を作成（簡易版、標準軸配置）
    fn from_radii(center: (T, T, T), a: T, b: T, c: T) -> Option<Self>
    where
        Self: Sized;

    /// 境界ボックスに内接する楕円体を作成
    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 球として楕円体を作成（a = b = c）
    fn new_sphere(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - EllipsoidalSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSolid3D基本プロパティ取得トレイト
pub trait EllipsoidalSolid3DProperties<T: Scalar> {
    // Phase 1: 基本プロパティ（7メソッド）

    /// 楕円体の中心点取得
    fn center(&self) -> (T, T, T);

    /// X軸方向の半径取得
    fn a_radius(&self) -> T;

    /// Y軸方向の半径取得
    fn b_radius(&self) -> T;

    /// Z軸方向の半径取得
    fn c_radius(&self) -> T;

    /// 参照軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 3軸の半径をタプルで取得 (a, b, c)
    fn radii(&self) -> (T, T, T);

    // Phase 2: 追加プロパティ（3メソッド）

    /// 球（a = b = c）かどうか判定
    fn is_sphere(&self) -> bool;

    /// 単位楕円体（全半径1）かどうか判定
    fn is_unit_ellipsoid(&self) -> bool;

    /// 原点中心かどうか判定
    fn is_centered_at_origin(&self) -> bool;
}

// ============================================================================
// 3. Measure Traits - EllipsoidalSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// EllipsoidalSolid3D測定機能トレイト
pub trait EllipsoidalSolid3DMeasure<T: Scalar> {
    // Phase 1: 基本測定（4メソッド）

    /// 楕円体ソリッドの体積を計算
    ///
    /// 体積 = (4/3)π × a × b × c
    fn volume(&self) -> T;

    /// 楕円体ソリッドの表面積を計算（Knudの近似式）
    ///
    /// S ≈ 4π × [(a^p × b^p + b^p × c^p + c^p × a^p) / 3]^(1/p)
    /// where p ≈ 1.6075
    fn surface_area(&self) -> T;

    /// 点が楕円体ソリッド内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点と楕円体ソリッド表面との距離を計算
    fn distance_to_surface(&self, point: (T, T, T)) -> T;

    // Phase 2: 追加測定（4メソッド）

    /// 楕円体の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));

    /// 指定点に最も近い表面上の点を取得
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);

    /// 点が楕円体表面上にあるか判定
    fn is_on_surface(&self, point: (T, T, T)) -> bool;

    /// 退化した楕円体かどうか判定（いずれかの半径が許容誤差以下）
    fn is_degenerate(&self) -> bool;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// EllipsoidalSolid3DのCore機能を統合するトレイト
///
/// Constructor/Properties/Measureの全機能を統合
pub trait EllipsoidalSolid3DCore<T: Scalar>:
    EllipsoidalSolid3DConstructor<T> + EllipsoidalSolid3DProperties<T> + EllipsoidalSolid3DMeasure<T>
{
}
