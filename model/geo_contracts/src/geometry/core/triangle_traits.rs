//! Triangle Core Traits - 三角形の基本機能トレイト
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! 3-5-4 パターン: Constructor(3+3) + Properties(5+3) + Measure(4+4)

use crate::Scalar;

/// Triangle2D Constructor トレイト（3+3メソッド）
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self>;

    /// 単位正三角形を生成
    fn unit_triangle() -> Self;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T); 3]) -> Option<Self>;

    /// 原点中心の正三角形を生成（辺の長さ指定）
    fn equilateral_at_origin(side_length: T) -> Self;

    /// 直角二等辺三角形を生成（原点、x軸、y軸上）
    fn right_isosceles(leg_length: T) -> Self;

    /// 頂点の順序を反転した三角形を作成
    fn reversed(&self) -> Self;
}

/// Triangle2D Properties トレイト（5+3メソッド）
pub trait Triangle2DProperties<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T);
}

pub trait Triangle2DDerived<T: Scalar> {
    /// 重心座標を取得
    fn centroid(&self) -> (T, T);

    /// 外心座標を取得
    fn circumcenter(&self) -> Option<(T, T)>;

    /// 内心座標を取得
    fn incenter(&self) -> (T, T);

    /// 外接円の半径を取得
    fn circumradius(&self) -> Option<T>;

    /// 内接円の半径を取得
    fn inradius(&self) -> T;

    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;

    /// 周囲長を計算
    fn perimeter(&self) -> T;

    /// 三角形が時計回りか判定
    fn is_clockwise(&self) -> bool;
}

pub trait Triangle2DContainment<T: Scalar> {
    /// 点が三角形内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait Triangle2DDistance<T: Scalar> {
    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T)) -> T;
}

/// Triangle2D Core トレイト（統合インターフェース）
pub trait Triangle2DCore<T: Scalar>: Triangle2DConstructor<T> + Triangle2DProperties<T> {}

/// Triangle3D Constructor トレイト（3+3メソッド）
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self>;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T, T); 3]) -> Option<Self>;

    /// xy平面上の単位正三角形
    fn unit_triangle_xy() -> Self;

    /// xz平面上の単位正三角形
    fn unit_triangle_xz() -> Self;

    /// yz平面上の単位正三角形
    fn unit_triangle_yz() -> Self;

    /// 頂点の順序を反転した三角形を作成（法線方向反転）
    fn reversed(&self) -> Self;
}

/// Triangle3D Properties トレイト（5+3メソッド）
pub trait Triangle3DProperties<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T, T);
}

pub trait Triangle3DDerived<T: Scalar> {
    /// 重心座標を取得
    fn centroid(&self) -> (T, T, T);

    /// 法線ベクトル（正規化済み）を取得
    fn normal(&self) -> (T, T, T);

    /// 外心座標を取得（三角形を含む平面上）
    fn circumcenter(&self) -> Option<(T, T, T)>;

    /// 外接円の半径を取得
    fn circumradius(&self) -> Option<T>;

    /// 内接円の半径を取得
    fn inradius(&self) -> T;

    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;

    /// 周囲長を計算
    fn perimeter(&self) -> T;

    /// 三角形が平面上にあるか判定
    fn is_planar(&self) -> bool;
}

pub trait Triangle3DContainment<T: Scalar> {
    /// 点が三角形内部にあるか判定（平面投影）
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait Triangle3DDistance<T: Scalar> {
    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

/// Triangle3D Core トレイト（統合インターフェース）
pub trait Triangle3DCore<T: Scalar>: Triangle3DConstructor<T> + Triangle3DProperties<T> {}

impl<T: Scalar, Triangle> Triangle2DCore<T> for Triangle where
    Triangle: Triangle2DConstructor<T> + Triangle2DProperties<T>
{
}

impl<T: Scalar, Triangle> Triangle3DCore<T> for Triangle where
    Triangle: Triangle3DConstructor<T> + Triangle3DProperties<T>
{
}
