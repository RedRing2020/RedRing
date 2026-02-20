//! NurbsCurve2D Core Traits - NURBS 2D曲線の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, from_control_points, unit_line）
//! - Properties: 5メソッド（degree, num_control_points, knot_vector, is_rational, dimension）
//! - Measure: 4メソッド（point_at, tangent_at, length, curvature_at）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - NurbsCurve2D生成機能（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve2D生成のためのConstructorトレイト
pub trait NurbsCurve2DConstructor<T: Scalar> {
    /// NURBS曲線を作成
    ///
    /// # Arguments
    /// * `control_points` - 制御点配列 [(x, y), ...]
    /// * `weights` - 重み配列（Noneの場合は非有理曲線）
    /// * `knot_vector` - ノットベクトル
    /// * `degree` - NURBS次数
    ///
    /// # Errors
    /// * 制御点数が次数+1未満の場合
    /// * ノットベクトルが無効な場合
    /// * 重み配列のサイズが制御点数と一致しない場合
    fn new(
        control_points: &[(T, T)],
        weights: Option<Vec<T>>,
        knot_vector: Vec<T>,
        degree: usize,
    ) -> Result<Self, String>
    where
        Self: Sized;

    /// 制御点のみから非有理NURBS曲線を作成（簡易コンストラクタ）
    ///
    /// 自動的にクランプされた均一ノットベクトルを生成
    fn from_control_points(control_points: &[(T, T)], degree: usize) -> Result<Self, String>
    where
        Self: Sized;

    /// 単位線分（(0,0)から(1,0)への直線）
    fn unit_line() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - NurbsCurve2D基本情報取得（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve2D基本プロパティ取得トレイト
pub trait NurbsCurve2DProperties<T: Scalar> {
    /// NURBS次数を取得
    fn degree(&self) -> usize;

    /// 制御点数を取得
    fn num_control_points(&self) -> usize;

    /// ノットベクトルへの参照を取得
    fn knot_vector(&self) -> &[T];

    /// 有理曲線かどうかを判定
    fn is_rational(&self) -> bool;

    /// 曲線の次元数を取得（2D曲線の場合は2）
    fn dimension(&self) -> usize {
        2
    }
}

// ============================================================================
// 3. Measure Traits - NurbsCurve2D測定・評価機能（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve2D測定・評価機能トレイト
pub trait NurbsCurve2DMeasure<T: Scalar> {
    /// パラメータ t での曲線上の点を計算
    ///
    /// # Arguments
    /// * `t` - パラメータ値（通常 [0, 1] の範囲）
    fn point_at(&self, t: T) -> (T, T);

    /// パラメータ t での接線ベクトルを計算
    ///
    /// # Arguments
    /// * `t` - パラメータ値
    fn tangent_at(&self, t: T) -> (T, T);

    /// 曲線の長さを計算
    ///
    /// 数値積分により近似計算
    fn length(&self) -> T;

    /// パラメータ t での曲率を計算
    ///
    /// # Arguments
    /// * `t` - パラメータ値
    fn curvature_at(&self, t: T) -> T;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// NurbsCurve2DのCore機能を統合するトレイト
pub trait NurbsCurve2DCore<T: Scalar>:
    NurbsCurve2DConstructor<T> + NurbsCurve2DProperties<T> + NurbsCurve2DMeasure<T>
{
}
