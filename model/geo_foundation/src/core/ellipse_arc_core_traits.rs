//! EllipseArc Core Traits - Phase 1 Minimal Implementation
//!
//! Foundation Pattern に基づく最小限のCore機能（12メソッド）
//! Core機能（Constructor/Properties/Measure）を3-5-4パターンで実装
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! 作成日: 2025年11月28日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - EllipseArc生成機能（3メソッド）
// ============================================================================

/// EllipseArc2D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait EllipseArc2DConstructor<T: Scalar> {
    /// 中心、長軸、短軸、回転角、角度範囲から楕円弧を作成
    fn new(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の単位楕円弧を作成（回転なし、0度から90度）
    fn unit_ellipse_arc() -> Self
    where
        Self: Sized;

    /// 指定した角度範囲の楕円弧を作成（回転なし）
    fn from_ellipse_and_angles(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc3D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait EllipseArc3DConstructor<T: Scalar> {
    /// 3D空間での楕円弧を作成（法線、長軸方向、角度範囲）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major: T,
        semi_minor: T,
        major_direction: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の楕円弧を作成
    fn xy_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 単位楕円弧をXY平面上に作成
    fn unit_ellipse_arc_xy() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - EllipseArc基本情報取得（5メソッド）
// ============================================================================

/// EllipseArc2D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait EllipseArc2DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
}

/// EllipseArc3D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait EllipseArc3DProperties<T: Scalar> {
    /// 中心点を取得
    fn center(&self) -> (T, T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - EllipseArc計量・関係演算機能（4メソッド）
// ============================================================================

/// EllipseArc2D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait EllipseArc2DMeasure<T: Scalar> {
    /// 楕円弧の長さ（測度）
    fn measure(&self) -> T;

    /// 開始点を取得
    fn start_point(&self) -> (T, T);

    /// 終了点を取得
    fn end_point(&self) -> (T, T);

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}

/// EllipseArc3D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait EllipseArc3DMeasure<T: Scalar> {
    /// 楕円弧の長さ（測度）
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

/// EllipseArc2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait EllipseArc2DCore<T: Scalar>:
    EllipseArc2DConstructor<T> + EllipseArc2DProperties<T> + EllipseArc2DMeasure<T>
{
}

/// EllipseArc3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait EllipseArc3DCore<T: Scalar>:
    EllipseArc3DConstructor<T> + EllipseArc3DProperties<T> + EllipseArc3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, E> EllipseArc2DCore<T> for E where
    E: EllipseArc2DConstructor<T> + EllipseArc2DProperties<T> + EllipseArc2DMeasure<T>
{
}

impl<T: Scalar, E> EllipseArc3DCore<T> for E where
    E: EllipseArc3DConstructor<T> + EllipseArc3DProperties<T> + EllipseArc3DMeasure<T>
{
}
