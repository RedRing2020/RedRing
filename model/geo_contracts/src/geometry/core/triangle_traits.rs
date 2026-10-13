//! Triangle 形状の契約定義
//!
//! face boundary access / boundary quantity / derived capability を分離する

use crate::{Direction3D, Point2D, Point3D, PointClassification, Scalar};

/// Triangle2D Constructor トレイト
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築する（3点が一直線上にある場合は `None`）
    fn new(a: Point2D<T>, b: Point2D<T>, c: Point2D<T>) -> Option<Self>;
}

/// Triangle2D の face boundary access
pub trait Triangle2DBoundaryAccess<T: Scalar> {
    fn vertex_a(&self) -> Point2D<T>;
    fn vertex_b(&self) -> Point2D<T>;
    fn vertex_c(&self) -> Point2D<T>;
}

pub trait Triangle2DContainment<T: Scalar> {
    /// 点が三角形の内部または境界上にあるか判定（境界を含む厳密な判定）
    fn contains_point(&self, point: Point2D<T>) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point2D<T>) -> PointClassification;
}

pub trait Triangle2DDistance<T: Scalar> {
    /// 点から三角形までの距離（内部の点は 0）
    fn distance_to_point(&self, point: Point2D<T>) -> T;
}

/// Triangle3D Constructor トレイト
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築する（3点が一直線上にある場合は `None`）
    fn new(a: Point3D<T>, b: Point3D<T>, c: Point3D<T>) -> Option<Self>;
}

/// Triangle3D の face boundary access
pub trait Triangle3DBoundaryAccess<T: Scalar> {
    fn vertex_a(&self) -> Point3D<T>;
    fn vertex_b(&self) -> Point3D<T>;
    fn vertex_c(&self) -> Point3D<T>;
}

pub trait Triangle3DDerived<T: Scalar> {
    /// 法線（AB × AC の向き）
    fn normal(&self) -> Direction3D<T>;
}

pub trait Triangle3DContainment<T: Scalar> {
    /// 点が三角形の内部または境界上にあるか判定（既定の距離トレランスで `classify_point` が `Outside` でない）
    fn contains_point(&self, point: Point3D<T>) -> bool;
    /// 領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: Point3D<T>) -> PointClassification;
}

pub trait Triangle3DDistance<T: Scalar> {
    /// 点から三角形までの最短距離
    fn distance_to_point(&self, point: Point3D<T>) -> T;
}
