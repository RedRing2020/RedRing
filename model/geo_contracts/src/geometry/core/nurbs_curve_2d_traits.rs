//! NurbsCurve2D の trait定義を capability taxonomy に沿って分離する。

use crate::Scalar;

/// NurbsCurve2D の生成 trait
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

/// NurbsCurve2D の定義パラメータ
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

/// NurbsCurve2D の評価
pub trait NurbsCurve2DEvaluation<T: Scalar> {
    /// パラメータ t での曲線上の点を計算
    fn point_at(&self, t: T) -> (T, T);

    /// checked 入口として使うための評価メソッド。
    ///
    /// デフォルト実装は後方互換のため `point_at` を `Some(...)` で包む
    /// unchecked ラッパであり、この trait定義だけでは domain 判定を行わない。
    ///
    /// 範囲外入力を `None` として扱う必要がある実装では、このメソッドを
    /// override して各型の parameter domain 判定を行うこと。
    fn point_at_checked(&self, t: T) -> Option<(T, T)> {
        Some(self.point_at(t))
    }

    /// パラメータ t での接線ベクトルを計算
    fn tangent_at(&self, t: T) -> (T, T);
}

/// NurbsCurve2D の派生量
pub trait NurbsCurve2DDerived<T: Scalar> {
    /// 曲線の長さを計算（数値積分により近似計算）
    fn length(&self) -> T;

    /// パラメータ t での曲率を計算
    fn curvature_at(&self, t: T) -> T;
}

/// NurbsCurve2D の互換 Core trait
pub trait NurbsCurve2DCore<T: Scalar>:
    NurbsCurve2DConstructor<T> + NurbsCurve2DProperties<T>
{
}

impl<T: Scalar, Curve> NurbsCurve2DCore<T> for Curve where
    Curve: NurbsCurve2DConstructor<T> + NurbsCurve2DProperties<T>
{
}
