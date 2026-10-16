//! NurbsCurve3D 形状の契約定義
//!
//! geo_nurbs が実装すべき 3 次元 NURBS 曲線の公開 trait。

use crate::{NurbsError, Point3D, Scalar, Vector3D};

pub trait NurbsCurve3DConstructor<T: Scalar>: Sized {
    /// 次数・ノットベクトル・制御点・重みから作成する
    ///
    /// 重みが `None` の場合は非有理（すべての重みが 1）。ノットベクトルの長さは 制御点の数 + 次数 + 1 で、
    /// 両端のノットは 次数 + 1 回以上重複する（クランプ）必要がある。
    fn new(
        degree: usize,
        knots: Vec<T>,
        control_points: &[Point3D<T>],
        weights: Option<Vec<T>>,
    ) -> Result<Self, NurbsError>;
}

pub trait NurbsCurve3DProperties<T: Scalar> {
    fn degree(&self) -> usize;
    fn knot_vector(&self) -> &[T];
    fn control_point_count(&self) -> usize;
    /// `index` 番目の制御点（範囲外は `None`）
    fn control_point(&self, index: usize) -> Option<Point3D<T>>;
    /// `index` 番目の制御点の重み（非有理の曲線では 1。範囲外は `None`）
    fn weight(&self, index: usize) -> Option<T>;
    /// 制御点の座標を並べた配列 `[x0, y0, z0, x1, y1, z1, ...]`（GPU 等への受け渡し用）
    fn coordinates(&self) -> &[T];
    /// 制御点の重みの配列（非有理の曲線では `None`。GPU 等への受け渡し用）
    fn weights(&self) -> Option<&[T]>;
    /// パラメータの定義域 `(開始, 終了)`
    fn parameter_domain(&self) -> (T, T);
}

pub trait NurbsCurve3DEvaluation<T: Scalar> {
    /// パラメータ `t` での曲線上の点
    ///
    /// 定義域から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    fn point_at_parameter(&self, t: T) -> Option<Point3D<T>>;

    /// パラメータ `t` での 1 階導関数（接線ベクトル）
    ///
    /// 定義域から距離トレランス分を超えて外れる `t` と、有限でない `t` は `None` を返す。
    fn derivative_at(&self, t: T) -> Option<Vector3D<T>>;
}

pub trait NurbsCurve3DDerived<T: Scalar> {
    /// 定義域全体の曲線の長さ（既定の距離トレランスまでの数値積分）
    fn length(&self) -> T;
}
