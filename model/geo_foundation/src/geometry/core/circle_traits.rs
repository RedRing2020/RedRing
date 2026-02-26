//! Circle Core Traits - Circle形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, new_with_ref_direction, unit_circle）
//! - Properties: 5メソッド（center, radius, ref_direction, diameter, dimension）
//! - Measure: 4メソッド（circumference, area, contains_point, distance_to_point）
//!
//! ## Phase 2 実装（標準機能追加）
//! - Constructor +3: from_center_and_point, from_three_points, centered_at_origin
//! - Properties +3: is_unit_circle, is_centered_at_origin, is_degenerate
//! - Measure +4: point_on_circumference, closest_point_to, point_at_parameter, distance_to_circle

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - Circle生成機能（Phase 1: 最小限）
// ============================================================================

/// Circle2D生成のためのConstructorトレイト
pub trait Circle2DConstructor<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本コンストラクタ（3メソッド）
    // ========================================================================

    /// 基本コンストラクタ（中心点、半径）
    /// デフォルトでX軸正方向を参照方向とする
    fn new(center: (T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// STEP準拠の参照方向を指定して作成
    fn new_with_ref_direction(center: (T, T), radius: T, ref_direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 単位円作成（原点中心、半径1）
    fn unit_circle() -> Self
    where
        Self: Sized;

    // ========================================================================
    // Phase 2: 標準コンストラクタ（+3メソッド）
    // ========================================================================

    /// 中心点と円周上の1点から円を作成
    fn from_center_and_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 3点から円を作成（3点を通る円）
    fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の円を作成
    fn centered_at_origin(radius: T) -> Option<Self>
    where
        Self: Sized;
}

/// Circle3D生成のためのConstructorトレイト
pub trait Circle3DConstructor<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本コンストラクタ（3メソッド）
    // ========================================================================

    /// 基本コンストラクタ（中心点、軸方向、半径）
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の円作成（Z軸が法線）
    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY平面単位円（原点中心、半径1）
    fn unit_circle_xy() -> Self
    where
        Self: Sized;

    // ========================================================================
    // Phase 2: 標準コンストラクタ（+5メソッド）
    // ========================================================================

    /// 中心点と円周上の1点から円を作成（3D）
    fn from_center_and_point(
        center: (T, T, T),
        axis: (T, T, T),
        point_on_circle: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 3点から円を作成（3点を通る円、自動的に平面を決定）
    fn from_three_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心のXY平面上の円を作成
    fn centered_at_origin_xy(radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XZ平面上の円作成（Y軸が法線）
    fn new_xz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// YZ平面上の円作成（X軸が法線）
    fn new_yz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - Circle基本情報取得（Phase 1: 最小限）
// ============================================================================

/// Circle2D基本プロパティ取得トレイト
pub trait Circle2DProperties<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本プロパティ（5メソッド）
    // ========================================================================

    /// 中心点取得
    fn center(&self) -> (T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T);

    /// 直径取得
    fn diameter(&self) -> T;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;

    // ========================================================================
    // Phase 2: 判定プロパティ（+3メソッド）
    // ========================================================================

    /// 単位円かどうか判定（半径が1）
    fn is_unit_circle(&self) -> bool;

    /// 原点中心かどうか判定
    fn is_centered_at_origin(&self) -> bool;

    /// 退化した円かどうか判定（半径がほぼ0）
    fn is_degenerate(&self) -> bool;
}

/// Circle3D基本プロパティ取得トレイト
pub trait Circle3DProperties<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本プロパティ（5メソッド）
    // ========================================================================

    /// 中心点取得
    fn center(&self) -> (T, T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 軸方向（法線）取得
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T, T);

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;

    // ========================================================================
    // Phase 2: 判定プロパティ（+4メソッド）
    // ========================================================================

    /// 単位円かどうか判定（半径が1）
    fn is_unit_circle(&self) -> bool;

    /// 原点中心かどうか判定
    fn is_centered_at_origin(&self) -> bool;

    /// 退化した円かどうか判定（半径がほぼ0）
    fn is_degenerate(&self) -> bool;

    /// XY平面上にあるかどうか判定
    fn is_on_xy_plane(&self) -> bool;
}

// ============================================================================
// 3. Measure Traits - Circle計量・関係演算機能（Phase 1: 最小限）
// ============================================================================

/// Circle2D計量・関係演算機能トレイト
pub trait Circle2DMeasure<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本計量（4メソッド）
    // ========================================================================

    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T)) -> T;

    // ========================================================================
    // Phase 2: 高度な計量・関係演算（+4メソッド）
    // ========================================================================

    /// 点が円周上にあるか判定（許容誤差考慮）
    fn point_on_circumference(&self, point: (T, T)) -> bool;

    /// 点に最も近い円周上の点を取得
    fn closest_point_to(&self, point: (T, T)) -> (T, T);

    /// パラメータt（0〜1）での円周上の点を取得
    /// t=0で参照方向、t=0.25で90度回転、t=0.5で180度、t=1.0で360度（=0度）
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 他の円との最短距離を計算
    fn distance_to_circle(&self, other: &Self) -> T;
}

/// Circle3D計量・関係演算機能トレイト
pub trait Circle3DMeasure<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本計量（4メソッド）
    // ========================================================================

    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定（平面上も考慮）
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    // ========================================================================
    // Phase 2: 高度な計量・関係演算（+4メソッド）
    // ========================================================================

    /// 点が円周上にあるか判定（3D空間、許容誤差考慮）
    fn point_on_circumference(&self, point: (T, T, T)) -> bool;

    /// 点に最も近い円周上の点を取得（3D）
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);

    /// パラメータt（0〜1）での円周上の点を取得（3D）
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 他の円との最短距離を計算（3D空間）
    fn distance_to_circle(&self, other: &Self) -> T;
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// Circle2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Circle2DCore<T: Scalar>:
    Circle2DConstructor<T> + Circle2DProperties<T> + Circle2DMeasure<T>
{
}

/// Circle3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Circle3DCore<T: Scalar>:
    Circle3DConstructor<T> + Circle3DProperties<T> + Circle3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, C> Circle2DCore<T> for C where
    C: Circle2DConstructor<T> + Circle2DProperties<T> + Circle2DMeasure<T>
{
}

impl<T: Scalar, C> Circle3DCore<T> for C where
    C: Circle3DConstructor<T> + Circle3DProperties<T> + Circle3DMeasure<T>
{
}
