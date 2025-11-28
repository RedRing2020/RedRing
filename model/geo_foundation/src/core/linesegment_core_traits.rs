//! LineSegment Core Traits - Phase 1 Minimal Implementation
//!
//! Foundation Pattern に基づく最小限のCore機能（12メソッド）
//! Core機能（Constructor/Properties/Measure）を3-5-4パターンで実装
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! 作成日: 2025年11月28日

use crate::Scalar;


// ============================================================================
// 1. Constructor Traits - LineSegment生成機能（3メソッド）
// ============================================================================

/// LineSegment2D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait LineSegment2DConstructor<T: Scalar> {
    /// 2つの点から線分を作成
    fn new(start: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 起点、方向、長さから線分を作成
    fn from_point_direction_length(start: (T, T), direction: (T, T), length: T) -> Option<Self>
    where
        Self: Sized;

    /// X軸方向の単位線分（原点から(1,0)まで）
    fn unit_x() -> Self
    where
        Self: Sized;
}

/// LineSegment3D生成のためのConstructorトレイト（Phase 1: 最小限）
pub trait LineSegment3DConstructor<T: Scalar> {
    /// 2つの3D点から線分を作成
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 起点、方向、長さから3D線分を作成
    fn from_point_direction_length(
        start: (T, T, T),
        direction: (T, T, T),
        length: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// X軸方向の単位線分（原点から(1,0,0)まで）
    fn unit_x() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - LineSegment基本情報取得（5メソッド）
// ============================================================================

/// LineSegment2D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait LineSegment2DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T);

    /// 終了点を取得
    fn end(&self) -> (T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T);

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// LineSegment3D基本プロパティ取得トレイト（Phase 1: 最小限）
pub trait LineSegment3DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T, T);

    /// 終了点を取得
    fn end(&self) -> (T, T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T, T);

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

// ============================================================================
// 3. Measure Traits - LineSegment計量・関係演算機能（4メソッド）
// ============================================================================

/// LineSegment2D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait LineSegment2DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}

/// LineSegment3D計量・関係演算機能トレイト（Phase 1: 最小限）
pub trait LineSegment3DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}



// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// LineSegment2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait LineSegment2DCore<T: Scalar>:
    LineSegment2DConstructor<T> + LineSegment2DProperties<T> + LineSegment2DMeasure<T>
{
}

/// LineSegment3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait LineSegment3DCore<T: Scalar>:
    LineSegment3DConstructor<T> + LineSegment3DProperties<T> + LineSegment3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, L> LineSegment2DCore<T> for L where
    L: LineSegment2DConstructor<T> + LineSegment2DProperties<T> + LineSegment2DMeasure<T>
{
}

impl<T: Scalar, L> LineSegment3DCore<T> for L where
    L: LineSegment3DConstructor<T> + LineSegment3DProperties<T> + LineSegment3DMeasure<T>
{
}
