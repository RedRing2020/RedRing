//! Triangle Core Traits - 三角形の基本機能トレイト
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! face boundary access / boundary quantity / derived capability を分離する

use crate::{PointClassification, Scalar};

/// Triangle2D Constructor トレイト（3+3メソッド）
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self>;
}

/// Triangle2D の face boundary access
pub trait Triangle2DBoundaryAccess<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T);
}

pub trait Triangle2DContainment<T: Scalar> {
    /// 点が三角形の内部または境界上にあるか判定（境界を含む厳密な判定）
    fn contains_point(&self, point: (T, T)) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: (T, T)) -> PointClassification;
}

pub trait Triangle2DDistance<T: Scalar> {
    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T)) -> T;
}

/// Triangle3D Constructor トレイト（3+3メソッド）
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self>;
}

/// Triangle3D の face boundary access
pub trait Triangle3DBoundaryAccess<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T, T);
}

pub trait Triangle3DDerived<T: Scalar> {
    /// 法線ベクトル（正規化済み）を取得
    fn normal(&self) -> (T, T, T);
}

pub trait Triangle3DContainment<T: Scalar> {
    /// 点が三角形の内部または境界上にあるか判定（既定の距離トレランスで `classify_point` が `Outside` でない）
    fn contains_point(&self, point: (T, T, T)) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: (T, T, T)) -> PointClassification;
}

pub trait Triangle3DDistance<T: Scalar> {
    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}
