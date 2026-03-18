//! NurbsCurve3D Core Traits - NURBS曲線の3つのCore機能統合
//!
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - NURBS Curve生成機能
// ============================================================================

/// NurbsCurve3D生成のためのConstructorトレイト
pub trait NurbsCurve3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（次数、ノット、制御点、重み）
    ///
    /// # 引数
    /// * `degree` - NURBS曲線の次数（1=線形、2=2次、3=3次）
    /// * `knots` - ノットベクトル
    /// * `control_points` - 制御点配列（x,y,z座標のタプル）
    /// * `weights` - 重み配列（Noneの場合は非有理曲線）
    fn new(
        degree: usize,
        knots: Vec<T>,
        control_points: Vec<(T, T, T)>,
        weights: Option<Vec<T>>,
    ) -> Result<Self, String>
    where
        Self: Sized;

    /// Bezier曲線として作成（クランプド・ノットベクトル使用）
    fn from_bezier(control_points: Vec<(T, T, T)>) -> Result<Self, String>
    where
        Self: Sized;

    /// 線分として作成（1次NURBS、2制御点）
    fn line_segment(start: (T, T, T), end: (T, T, T)) -> Result<Self, String>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - NURBS Curve基本情報取得
// ============================================================================

/// NurbsCurve3D基本プロパティ取得トレイト
pub trait NurbsCurve3DProperties<T: Scalar> {
    /// NURBS曲線の次数を取得
    fn degree(&self) -> usize;

    /// ノットベクトルへの参照を取得
    fn knot_vector(&self) -> &[T];

    /// 制御点の数を取得
    fn control_points_count(&self) -> usize;

    /// 重み配列への参照を取得
    fn weights(&self) -> Option<&[T]>;

    /// 有理曲線かどうかを判定
    fn is_rational(&self) -> bool;

    /// パラメータ定義域を取得 (u_min, u_max)
    fn parameter_domain(&self) -> (T, T);

    /// 全制御点座標のflattened配列への参照を取得
    ///
    /// [x0, y0, z0, x1, y1, z1, ...] の形式のスライス参照
    fn coordinates(&self) -> &[T];
}

// ============================================================================
// 3. Measure Traits - NURBS Curve計量・評価機能
// ============================================================================

/// NurbsCurve3D計量・評価トレイト
pub trait NurbsCurve3DMeasure<T: Scalar> {
    /// 指定されたパラメータ範囲の曲線長を計算（数値積分）
    fn arc_length(&self, u_start: T, u_end: T, tolerance: T) -> T;

    /// 曲線全体の長さを計算
    fn arc_length_total(&self, tolerance: T) -> T;

    /// 指定された曲線長に対応するパラメータ値を計算
    fn parameter_at_length(&self, arc_length: T, tolerance: T) -> Option<T>;

    /// 指定されたパラメータ値における曲線上の点を評価
    fn evaluate(&self, u: T) -> Option<(T, T, T)>;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// NurbsCurve3DのCore機能を統合するトレイト
pub trait NurbsCurve3DCore<T: Scalar>:
    NurbsCurve3DConstructor<T> + NurbsCurve3DProperties<T> + NurbsCurve3DMeasure<T>
{
}
