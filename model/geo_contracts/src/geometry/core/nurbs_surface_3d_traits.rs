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

    /// 制御点グリッドから非有理NURBSサーフェスを作成（簡易コンストラクタ）
    fn from_control_points(
        control_points: Vec<Vec<(T, T, T)>>,
        u_degree: usize,
        v_degree: usize,
    ) -> Result<Self, String>
    where
        Self: Sized;

    /// 単位平面サーフェス（XY平面上の1×1正方形）
    fn unit_plane() -> Self
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

    /// 有理サーフェスかどうかを判定
    fn is_rational(&self) -> bool;

    /// 全制御点座標のflattened配列への参照を取得（u方向優先でflatten）
    fn coordinates(&self) -> &[T];

    /// 全重みのflattened配列への参照を取得（u方向優先でflatten）
    fn weights(&self) -> Option<&[T]>;
}

/// NurbsSurface3D の評価
pub trait NurbsSurface3DEvaluation<T: Scalar> {
    /// パラメータ座標(u, v)でのサーフェス上の点を計算
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// checked 入口。
    ///
    /// このデフォルト実装は互換維持のための暫定ラッパであり、domain 判定を行わず
    /// `Some(self.point_at_uv(u, v))` をそのまま返す。
    ///
    /// 範囲外入力を失敗として扱いたい実装は、このメソッドを override して
    /// u/v の有効範囲を判定し、範囲外では `None` を返すことを想定している。
    fn point_at_uv_checked(&self, u: T, v: T) -> Option<(T, T, T)> {
        Some(self.point_at_uv(u, v))
    }

    /// パラメータ座標(u, v)での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での接線ベクトル(du, dv)を計算
    fn tangent_vectors_at(&self, u: T, v: T) -> ((T, T, T), (T, T, T));
}

/// NurbsSurface3D の派生量
pub trait NurbsSurface3DDerived<T: Scalar> {
    /// サーフェスの表面積を計算（数値積分により近似計算）
    fn surface_area(&self) -> T;
}

/// NurbsSurface3D の互換 Core trait
pub trait NurbsSurface3DCore<T: Scalar>:
    NurbsSurface3DConstructor<T> + NurbsSurface3DProperties<T>
{
}

impl<T: Scalar, Surface> NurbsSurface3DCore<T> for Surface where
    Surface: NurbsSurface3DConstructor<T> + NurbsSurface3DProperties<T>
{
}
