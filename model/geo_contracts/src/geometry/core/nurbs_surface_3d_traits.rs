//! NurbsSurface3D 形状の契約定義
//!
//! geo_nurbs が実装すべき NURBS 曲面の公開 trait。

use crate::{Direction3D, NurbsError, Point3D, Scalar};

pub trait NurbsSurface3DConstructor<T: Scalar>: Sized {
    /// u・v 方向の次数とノットベクトル、制御点の格子、重みの格子から作成する
    ///
    /// `control_points[i][j]` は u 方向の `i` 番目・v 方向の `j` 番目の制御点。重みの格子も同じ並びで、
    /// `None` の場合は非有理（すべての重みが 1）。各方向のノットベクトルの長さは
    /// その方向の制御点の数 + 次数 + 1 で、両端のノットは 次数 + 1 回以上重複する（クランプ）必要がある。
    fn new(
        u_degree: usize,
        v_degree: usize,
        u_knots: Vec<T>,
        v_knots: Vec<T>,
        control_points: &[Vec<Point3D<T>>],
        weights: Option<Vec<Vec<T>>>,
    ) -> Result<Self, NurbsError>;
}

pub trait NurbsSurface3DProperties<T: Scalar> {
    fn u_degree(&self) -> usize;
    fn v_degree(&self) -> usize;
    /// u 方向の制御点の数
    fn u_count(&self) -> usize;
    /// v 方向の制御点の数
    fn v_count(&self) -> usize;
    fn u_knots(&self) -> &[T];
    fn v_knots(&self) -> &[T];
    /// u 方向の `u` 番目・v 方向の `v` 番目の制御点（範囲外は `None`）
    fn control_point(&self, u: usize, v: usize) -> Option<Point3D<T>>;
    /// u 方向の `u` 番目・v 方向の `v` 番目の制御点の重み（非有理の曲面では 1。範囲外は `None`）
    fn weight(&self, u: usize, v: usize) -> Option<T>;
    /// 制御点の座標を u 方向優先で並べた配列 `[x00, y00, z00, x01, ...]`（GPU 等への受け渡し用）
    fn coordinates(&self) -> &[T];
    /// 制御点の重みを u 方向優先で並べた配列（非有理の曲面では `None`。GPU 等への受け渡し用）
    fn weights(&self) -> Option<&[T]>;
    /// パラメータの定義域 `((u の開始, u の終了), (v の開始, v の終了))`
    fn parameter_domain(&self) -> ((T, T), (T, T));
}

pub trait NurbsSurface3DEvaluation<T: Scalar> {
    /// パラメータ `(u, v)` での曲面上の点
    ///
    /// 定義域から距離トレランス分を超えて外れる値と、有限でない値は `None` を返す。
    fn point_at_uv(&self, u: T, v: T) -> Option<Point3D<T>>;

    /// パラメータ `(u, v)` での法線（∂u × ∂v の向き）
    ///
    /// 偏導関数が平行または 0 になる点（球の極・縮退した辺など）では、定義域の内側から近づいたときの法線の
    /// 極限を 2 階の偏導関数から求めて返す。曲面全体が縮退していて法線が定まらない場合と、
    /// 定義域の外の値は `None`。
    fn normal_at(&self, u: T, v: T) -> Option<Direction3D<T>>;
}
