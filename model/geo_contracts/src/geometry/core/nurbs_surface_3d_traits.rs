//! NurbsSurface3D の trait定義を capability taxonomy に沿って分離する。

use crate::Scalar;

/// NurbsSurface3D の生成 trait
pub trait NurbsSurface3DConstructor<T: Scalar> {
    /// NURBSサーフェスを作成
    ///
    /// # Arguments
    /// * `control_points` - 制御点の2次元グリッド [u_count][v_count]
    /// * `weights` - 重みの2次元グリッド（Noneの場合は非有理サーフェス）
    /// * `u_knots` - u方向ノットベクトル
    /// * `v_knots` - v方向ノットベクトル
    /// * `u_degree` - u方向次数
    /// * `v_degree` - v方向次数
    fn new(
        control_points: Vec<Vec<(T, T, T)>>,
        weights: Option<Vec<Vec<T>>>,
        u_knots: Vec<T>,
        v_knots: Vec<T>,
        u_degree: usize,
        v_degree: usize,
    ) -> Result<Self, String>
    where
        Self: Sized;
}

/// NurbsSurface3D の定義パラメータ
pub trait NurbsSurface3DProperties<T: Scalar> {
    /// u方向のNURBS次数を取得
    fn u_degree(&self) -> usize;

    /// v方向のNURBS次数を取得
    fn v_degree(&self) -> usize;

    /// u方向の制御点数を取得
    fn u_count(&self) -> usize;

    /// v方向の制御点数を取得
    fn v_count(&self) -> usize;

    /// u方向ノットベクトルへの参照を取得
    fn u_knots(&self) -> &[T];

    /// v方向ノットベクトルへの参照を取得
    fn v_knots(&self) -> &[T];

    /// 全制御点座標のflattened配列への参照を取得（u方向優先でflatten）
    fn coordinates(&self) -> &[T];

    /// 全重みのflattened配列への参照を取得（u方向優先でflatten）
    fn weights(&self) -> Option<&[T]>;
}

/// NurbsSurface3D の評価
pub trait NurbsSurface3DEvaluation<T: Scalar> {
    /// パラメータ座標(u, v)でのサーフェス上の点を計算
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> (T, T, T);
}

/// NurbsSurface3D の派生量
pub trait NurbsSurface3DDerived<T: Scalar> {
    /// サーフェスの表面積を計算（数値積分により近似計算）
    fn surface_area(&self) -> T;
}
