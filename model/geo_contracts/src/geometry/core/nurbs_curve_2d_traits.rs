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
}

/// NurbsCurve2D の定義パラメータ
pub trait NurbsCurve2DProperties<T: Scalar> {
    /// NURBS次数を取得
    fn degree(&self) -> usize;

    /// 制御点数を取得
    fn num_control_points(&self) -> usize;

    /// ノットベクトルへの参照を取得
    fn knot_vector(&self) -> &[T];

    /// 曲線の次元数を取得（2D曲線の場合は2）
    fn dimension(&self) -> usize {
        2
    }
}

/// NurbsCurve2D の評価
pub trait NurbsCurve2DEvaluation<T: Scalar> {
    /// パラメータ t での曲線上の点を計算
    fn point_at(&self, t: T) -> (T, T);
}

/// NurbsCurve2D の派生量
pub trait NurbsCurve2DDerived<T: Scalar> {
    /// 曲線の長さを計算（数値積分により近似計算）
    fn length(&self) -> T;
}
