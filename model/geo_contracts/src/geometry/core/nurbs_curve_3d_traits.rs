//! NurbsCurve3D の trait定義を capability taxonomy に沿って分離する。

use crate::Scalar;

/// NurbsCurve3D の生成 trait
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

/// NurbsCurve3D の定義パラメータ
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

/// NurbsCurve3D の派生量
pub trait NurbsCurve3DDerived<T: Scalar> {
    /// 指定されたパラメータ範囲の曲線長を計算（数値積分）
    fn arc_length(&self, u_start: T, u_end: T, tolerance: T) -> T;

    /// 曲線全体の長さを計算
    fn arc_length_total(&self, tolerance: T) -> T;
}

/// NurbsCurve3D の評価
pub trait NurbsCurve3DEvaluation<T: Scalar> {
    /// 指定された曲線長に対応するパラメータ値を計算
    fn parameter_at_length(&self, arc_length: T, tolerance: T) -> Option<T>;

    /// 指定されたパラメータ値における曲線上の点を評価
    fn evaluate(&self, u: T) -> Option<(T, T, T)>;
}

/// NurbsCurve3D の互換 Core trait
pub trait NurbsCurve3DCore<T: Scalar>:
    NurbsCurve3DConstructor<T> + NurbsCurve3DProperties<T>
{
}

impl<T: Scalar, Curve> NurbsCurve3DCore<T> for Curve where
    Curve: NurbsCurve3DConstructor<T> + NurbsCurve3DProperties<T>
{
}
