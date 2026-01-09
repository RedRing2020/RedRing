//! NURBS Curve Core Traits - NURBS曲線の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, from_bezier, line_segment）
//! - Properties: 6メソッド（degree, knot_vector, control_points_count, weights, is_rational, parameter_domain）
//! - Measure: 4メソッド（arc_length, arc_length_total, parameter_at_length, evaluate）
//!
//! ## 実装例
//! ```ignore
//! use geo_nurbs::NurbsCurve3D;
//! use geo_foundation::{NurbsCurve3DConstructor, NurbsCurve3DProperties, NurbsCurve3DMeasure};
//! 
//! // Constructor Trait経由で作成
//! let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
//!     (0.0, 0.0, 0.0),
//!     (1.0, 0.0, 0.0)
//! ).unwrap();
//! 
//! // Properties Trait経由で情報取得
//! let deg = <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::degree(&curve);
//! let is_rat = <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::is_rational(&curve);
//! 
//! // Measure Trait経由で計量
//! let point = <NurbsCurve3D<f64> as NurbsCurve3DMeasure<f64>>::evaluate(&curve, 0.5).unwrap();
//! let length = <NurbsCurve3D<f64> as NurbsCurve3DMeasure<f64>>::arc_length_total(&curve, 1e-6);
//! ```
//!
//! 作成日: 2026年1月9日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - NURBS Curve生成機能（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve3D生成のためのConstructorトレイト
///
/// NURBS曲線の基本的な生成方法を提供します。
/// Phase 1では最小限の3つのコンストラクタのみ実装。
pub trait NurbsCurve3DConstructor<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本コンストラクタ（3メソッド）
    // ========================================================================

    /// 基本コンストラクタ（次数、ノット、制御点、重み）
    ///
    /// # 引数
    /// * `degree` - NURBS曲線の次数（1=線形、2=2次、3=3次）
    /// * `knots` - ノットベクトル
    /// * `control_points` - 制御点配列（x,y,z座標のタプル）
    /// * `weights` - 重み配列（Noneの場合は非有理曲線）
    ///
    /// # エラー
    /// * 制御点数が次数+1未満の場合
    /// * ノットベクトルが無効な場合
    /// * 重み配列のサイズが制御点数と一致しない場合
    ///
    /// # 戻り値
    /// 成功時は新しいNURBS曲線、失敗時はエラーメッセージ
    fn new(
        degree: usize,
        knots: Vec<T>,
        control_points: Vec<(T, T, T)>,
        weights: Option<Vec<T>>,
    ) -> Result<Self, String>
    where
        Self: Sized;

    /// Bezier曲線として作成（クランプド・ノットベクトル使用）
    ///
    /// n個の制御点からn-1次のBezier曲線を作成します。
    /// ノットベクトルは自動的にクランプド形式で生成されます。
    ///
    /// # 引数
    /// * `control_points` - Bezier曲線の制御点（x,y,z座標のタプル）
    ///
    /// # エラー
    /// * 制御点数が2未満の場合
    ///
    /// # 戻り値
    /// 成功時は新しいBezier曲線、失敗時はエラーメッセージ
    fn from_bezier(control_points: Vec<(T, T, T)>) -> Result<Self, String>
    where
        Self: Sized;

    /// 線分として作成（1次NURBS、2制御点）
    ///
    /// 2つの点を結ぶ直線をNURBS曲線として表現します。
    /// 次数1、ノットベクトル[0,0,1,1]で作成されます。
    ///
    /// # 引数
    /// * `start` - 始点（x,y,z座標のタプル）
    /// * `end` - 終点（x,y,z座標のタプル）
    ///
    /// # エラー
    /// * 始点と終点が一致する場合
    ///
    /// # 戻り値
    /// 成功時は新しい線分NURBS曲線、失敗時はエラーメッセージ
    fn line_segment(start: (T, T, T), end: (T, T, T)) -> Result<Self, String>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - NURBS Curve基本情報取得（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve3D基本プロパティ取得トレイト
///
/// NURBS曲線の構造的な情報（次数、ノット、制御点数等）を取得します。
pub trait NurbsCurve3DProperties<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本プロパティ（6メソッド）
    // ========================================================================

    /// NURBS曲線の次数を取得
    ///
    /// # 戻り値
    /// 曲線の次数（1=線形、2=2次、3=3次）
    fn degree(&self) -> usize;

    /// ノットベクトルへの参照を取得
    ///
    /// # 戻り値
    /// ノットベクトルのスライス参照
    fn knot_vector(&self) -> &[T];

    /// 制御点の数を取得
    ///
    /// # 戻り値
    /// 制御点の総数
    fn control_points_count(&self) -> usize;

    /// 重み配列への参照を取得
    ///
    /// # 戻り値
    /// 有理曲線の場合は`Some(&[T])`、非有理曲線の場合は`None`
    fn weights(&self) -> Option<&[T]>;

    /// 有理曲線かどうかを判定
    ///
    /// # 戻り値
    /// 重み配列が存在する場合は`true`、そうでない場合は`false`
    fn is_rational(&self) -> bool;

    /// パラメータ定義域を取得
    ///
    /// # 戻り値
    /// (u_min, u_max) のタプル
    fn parameter_domain(&self) -> (T, T);
}

// ============================================================================
// 3. Measure Traits - NURBS Curve計量・評価機能（Phase 1: 最小限）
// ============================================================================

/// NurbsCurve3D計量・評価トレイト
///
/// NURBS曲線上の点の評価、曲線長の計算など、計量関連の機能を提供します。
pub trait NurbsCurve3DMeasure<T: Scalar> {
    // ========================================================================
    // Phase 1: 基本計量（4メソッド）
    // ========================================================================

    /// 指定されたパラメータ範囲の曲線長を計算（数値積分）
    ///
    /// # 引数
    /// * `u_start` - 開始パラメータ
    /// * `u_end` - 終了パラメータ
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// 曲線長の近似値
    ///
    /// # 注意
    /// 数値積分による近似値です。高精度が必要な場合はtoleranceを小さくしてください。
    fn arc_length(&self, u_start: T, u_end: T, tolerance: T) -> T;

    /// 曲線全体の長さを計算
    ///
    /// # 引数
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// 曲線全体の長さの近似値
    fn arc_length_total(&self, tolerance: T) -> T;

    /// 指定された曲線長に対応するパラメータ値を計算
    ///
    /// # 引数
    /// * `arc_length` - 曲線長
    /// * `tolerance` - 許容誤差
    ///
    /// # 戻り値
    /// パラメータ値が見つかった場合は`Some(u)`、そうでない場合は`None`
    fn parameter_at_length(&self, arc_length: T, tolerance: T) -> Option<T>;

    /// 指定されたパラメータ値における曲線上の点を評価
    ///
    /// # 引数
    /// * `u` - パラメータ値（parameter_domain()の範囲内である必要がある）
    ///
    /// # 戻り値
    /// 曲線上の点の座標（x, y, z）のタプル
    /// パラメータが範囲外の場合は`None`
    fn evaluate(&self, u: T) -> Option<(T, T, T)>;
}
