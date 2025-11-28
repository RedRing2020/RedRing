//! Triangle Core Traits - 三角形の基本機能トレイト
//!
//! Foundation Pattern Phase 1 実装
//! 3-5-4 パターン: Constructor(3) + Properties(5) + Measure(4)
//!
//! 作成日: 2025年11月28日

use crate::Scalar;

// ============================================================================
// Triangle2D Core Traits
// ============================================================================

/// Triangle2D Constructor トレイト（3メソッド）
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self>;

    /// 単位正三角形を生成
    fn unit_triangle() -> Self;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T); 3]) -> Option<Self>;
}

/// Triangle2D Properties トレイト（5メソッド）
pub trait Triangle2DProperties<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T);

    /// 重心座標を取得
    fn centroid(&self) -> (T, T);

    /// 外心座標を取得
    fn circumcenter(&self) -> Option<(T, T)>;
}

/// Triangle2D Measure トレイト（4メソッド）
pub trait Triangle2DMeasure<T: Scalar> {
    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;
}

/// Triangle2D Core トレイト（統合インターフェース）
pub trait Triangle2DCore<T: Scalar>:
    Triangle2DConstructor<T> + Triangle2DProperties<T> + Triangle2DMeasure<T>
{
}

// ============================================================================
// Triangle3D Core Traits
// ============================================================================

/// Triangle3D Constructor トレイト（3メソッド）
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self>;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T, T); 3]) -> Option<Self>;

    /// xy平面上の単位正三角形
    fn unit_triangle_xy() -> Self;
}

/// Triangle3D Properties トレイト（5メソッド）
pub trait Triangle3DProperties<T: Scalar> {
    /// 頂点A座標を取得
    fn vertex_a(&self) -> (T, T, T);

    /// 頂点B座標を取得
    fn vertex_b(&self) -> (T, T, T);

    /// 頂点C座標を取得
    fn vertex_c(&self) -> (T, T, T);

    /// 重心座標を取得
    fn centroid(&self) -> (T, T, T);

    /// 法線ベクトル（正規化済み）を取得
    fn normal(&self) -> (T, T, T);
}

/// Triangle3D Measure トレイト（4メソッド）
pub trait Triangle3DMeasure<T: Scalar> {
    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;
}

/// Triangle3D Core トレイト（統合インターフェース）
pub trait Triangle3DCore<T: Scalar>:
    Triangle3DConstructor<T> + Triangle3DProperties<T> + Triangle3DMeasure<T>
{
}
