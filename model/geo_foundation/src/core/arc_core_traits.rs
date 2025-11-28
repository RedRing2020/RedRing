//! Arc Core Traits - Phase 1 Minimal Implementation
//!
//! Foundation Pattern に基づく最小限のCore機能（12メソッド）
//! Core機能（Constructor/Properties/Measure）を3-5-4パターンで実装
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! 作成日: 2025年11月28日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - Arc生成機能（3メソッド）
// ============================================================================

/// Arc2D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait Arc2DConstructor<T: Scalar> {
    /// 中心、半径、角度から円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点 (x, y)
    /// * `radius` - 半径（正の値）
    /// * `start_angle` - 開始角度（ラジアン）
    /// * `end_angle` - 終了角度（ラジアン）
    fn new(center: (T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    /// 3点から円弧を作成
    ///
    /// 3点を通る円弧を生成（開始点→中間点→終了点の順）
    fn from_three_points(start: (T, T), mid: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 半円を作成（開始角度0、終了角度π）
    fn semicircle(center: (T, T), radius: T) -> Self
    where
        Self: Sized;
}

/// Arc3D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait Arc3DConstructor<T: Scalar> {
    /// 中心、半径、法線、角度から円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点 (x, y, z)
    /// * `radius` - 半径（正の値）
    /// * `normal` - 円弧平面の法線ベクトル
    /// * `start_angle` - 開始角度（ラジアン）
    /// * `end_angle` - 終了角度（ラジアン）
    fn new(
        center: (T, T, T),
        radius: T,
        normal: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の円弧を作成
    ///
    /// Z軸を法線とする円弧を生成
    fn xy_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self>
    where
        Self: Sized;

    /// 3点から3D円弧を作成
    ///
    /// 3点を通る円弧を生成（開始点→中間点→終了点の順）
    fn from_three_points(start: (T, T, T), mid: (T, T, T), end: (T, T, T)) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - Arc基本情報取得（5メソッド）
// ============================================================================

/// Arc2D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait Arc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T);

    /// 半径を取得
    fn radius(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// Arc3D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait Arc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T, T);

    /// 半径を取得
    fn radius(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

// ============================================================================
// 3. Measure Traits - Arc計量・関係演算機能（4メソッド）
// ============================================================================

/// Arc2D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait Arc2DMeasure<T: Scalar> {
    /// 円弧の長さ（測度）
    fn measure(&self) -> T;

    /// 開始点を取得
    fn start_point(&self) -> (T, T);

    /// 終了点を取得
    fn end_point(&self) -> (T, T);

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}

/// Arc3D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait Arc3DMeasure<T: Scalar> {
    /// 円弧の長さ（測度）
    fn measure(&self) -> T;

    /// 開始点を取得
    fn start_point(&self) -> (T, T, T);

    /// 終了点を取得
    fn end_point(&self) -> (T, T, T);

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// Arc2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Arc2DCore<T: Scalar>:
    Arc2DConstructor<T> + Arc2DProperties<T> + Arc2DMeasure<T>
{
}

/// Arc3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Arc3DCore<T: Scalar>:
    Arc3DConstructor<T> + Arc3DProperties<T> + Arc3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, A> Arc2DCore<T> for A where
    A: Arc2DConstructor<T> + Arc2DProperties<T> + Arc2DMeasure<T>
{
}

impl<T: Scalar, A> Arc3DCore<T> for A where
    A: Arc3DConstructor<T> + Arc3DProperties<T> + Arc3DMeasure<T>
{
}
