//! Circle Core Traits - Circle形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - Circle生成機能
// ============================================================================

/// Circle2D生成のためのConstructorトレイト
pub trait Circle2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点と半径）
    fn new(center: (T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 3点から外接円を作成
    fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 中心点と円周上の点から作成
    fn from_center_point(center: (T, T), point_on_circle: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 単位円作成（原点中心、半径1）
    fn unit_circle() -> Self
    where
        Self: Sized;
}

/// Circle3D生成のためのConstructorトレイト
pub trait Circle3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、軸方向、半径）
    fn new(center: (T, T, T), axis: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 完全な座標系で作成
    fn new_with_ref_direction(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の円作成
    fn new_xy_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XZ平面上の円作成
    fn new_xz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// YZ平面上の円作成
    fn new_yz_plane(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// XY平面単位円
    fn unit_circle_xy() -> Self
    where
        Self: Sized;

    /// XZ平面単位円
    fn unit_circle_xz() -> Self
    where
        Self: Sized;

    /// YZ平面単位円
    fn unit_circle_yz() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - Circle基本情報取得
// ============================================================================

/// Circle2D基本プロパティ取得トレイト
pub trait Circle2DProperties<T: Scalar> {
    /// 中心点取得
    fn center(&self) -> (T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T);

    /// 直径取得
    fn diameter(&self) -> T;

    /// 点円（半径がゼロに近い）かどうか
    fn is_point(&self) -> bool;

    /// 単位円かどうか
    fn is_unit_circle(&self) -> bool;

    /// 原点中心かどうか
    fn is_centered_at_origin(&self) -> bool;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// Circle3D基本プロパティ取得トレイト
pub trait Circle3DProperties<T: Scalar> {
    /// 中心点取得
    fn center(&self) -> (T, T, T);

    /// 半径取得
    fn radius(&self) -> T;

    /// 軸方向（法線）取得
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得
    fn ref_direction(&self) -> (T, T, T);

    /// 法線ベクトル取得（後方互換性）
    fn normal(&self) -> (T, T, T);

    /// 直径取得
    fn diameter(&self) -> T;

    /// 点円（半径がゼロに近い）かどうか
    fn is_point(&self) -> bool;

    /// 単位円かどうか
    fn is_unit_circle(&self) -> bool;

    /// 原点中心かどうか
    fn is_centered_at_origin(&self) -> bool;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

// ============================================================================
// 3. Measure Traits - Circle計量・関係演算機能
// ============================================================================

/// Circle2D計量・関係演算機能トレイト
pub trait Circle2DMeasure<T: Scalar> {
    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点が円周上にあるか判定
    fn point_on_circumference(&self, point: (T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点における円のパラメータを取得
    fn parameter_at_point(&self, point: (T, T)) -> Option<T>;

    /// 2つの円の距離
    fn distance_to_circle(&self, other: &Self) -> T;

    /// 点に最も近い円周上の点を取得
    fn closest_point_to(&self, point: (T, T)) -> (T, T);
}

/// Circle3D計量・関係演算機能トレイト
pub trait Circle3DMeasure<T: Scalar> {
    /// 円周の長さ
    fn circumference(&self) -> T;

    /// 円の面積
    fn area(&self) -> T;

    /// 点が円内部にあるか判定（平面上も考慮）
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点が円周上にあるか判定
    fn point_on_circumference(&self, point: (T, T, T)) -> bool;

    /// 点から円周への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 点における円のパラメータを取得
    fn parameter_at_point(&self, point: (T, T, T)) -> Option<T>;

    /// 2つの円の距離
    fn distance_to_circle(&self, other: &Self) -> T;

    /// 点に最も近い円周上の点を取得
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
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
