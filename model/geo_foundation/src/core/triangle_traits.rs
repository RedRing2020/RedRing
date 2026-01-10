//! Triangle Core Traits - 三角形の基本機能トレイト
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! 3-5-4 パターン: Constructor(3+3) + Properties(5+3) + Measure(4+4)
//!
//! 作成日: 2025年11月28日
//! 最終更新日: 2025年11月29日（Phase 2追加）

use crate::Scalar;

// ============================================================================
// Triangle2D Core Traits
// ============================================================================

/// Triangle2D Constructor トレイト（3+3メソッド）
pub trait Triangle2DConstructor<T: Scalar>: Sized {
    // ========== Phase 1 実装 ==========
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T), b: (T, T), c: (T, T)) -> Option<Self>;

    /// 単位正三角形を生成
    fn unit_triangle() -> Self;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T); 3]) -> Option<Self>;

    // ========== Phase 2 実装 ==========
    /// 原点中心の正三角形を生成（辺の長さ指定）
    fn equilateral_at_origin(side_length: T) -> Self;

    /// 直角二等辺三角形を生成（原点、x軸、y軸上）
    fn right_isosceles(leg_length: T) -> Self;

    /// 頂点の順序を反転した三角形を作成
    fn reversed(&self) -> Self;
}

/// Triangle2D Properties トレイト（5+3メソッド）
pub trait Triangle2DProperties<T: Scalar> {
    // ========== Phase 1 実装 ==========
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

    // ========== Phase 2 実装 ==========
    /// 内心座標を取得
    fn incenter(&self) -> (T, T);

    /// 外接円の半径を取得
    fn circumradius(&self) -> Option<T>;

    /// 内接円の半径を取得
    fn inradius(&self) -> T;
}

/// Triangle2D Measure トレイト（4+4メソッド）
pub trait Triangle2DMeasure<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;

    // ========== Phase 2 実装 ==========
    /// 周囲長を計算
    fn perimeter(&self) -> T;

    /// 点が三角形内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 三角形が時計回りか判定
    fn is_clockwise(&self) -> bool;

    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T)) -> T;
}

/// Triangle2D Core トレイト（統合インターフェース）
pub trait Triangle2DCore<T: Scalar>:
    Triangle2DConstructor<T> + Triangle2DProperties<T> + Triangle2DMeasure<T>
{
}

// ============================================================================
// Triangle3D Core Traits
// ============================================================================

/// Triangle3D Constructor トレイト（3+3メソッド）
pub trait Triangle3DConstructor<T: Scalar>: Sized {
    // ========== Phase 1 実装 ==========
    /// 3点から三角形を構築
    ///
    /// 退化した三角形（3点が一直線上）の場合は None を返す
    fn new(a: (T, T, T), b: (T, T, T), c: (T, T, T)) -> Option<Self>;

    /// 配列から三角形を構築
    fn from_array(points: [(T, T, T); 3]) -> Option<Self>;

    /// xy平面上の単位正三角形
    fn unit_triangle_xy() -> Self;

    // ========== Phase 2 実装 ==========
    /// xz平面上の単位正三角形
    fn unit_triangle_xz() -> Self;

    /// yz平面上の単位正三角形
    fn unit_triangle_yz() -> Self;

    /// 頂点の順序を反転した三角形を作成（法線方向反転）
    fn reversed(&self) -> Self;
}

/// Triangle3D Properties トレイト（5+3メソッド）
pub trait Triangle3DProperties<T: Scalar> {
    // ========== Phase 1 実装 ==========
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

    // ========== Phase 2 実装 ==========
    /// 外心座標を取得（三角形を含む平面上）
    fn circumcenter(&self) -> Option<(T, T, T)>;

    /// 外接円の半径を取得
    fn circumradius(&self) -> Option<T>;

    /// 内接円の半径を取得
    fn inradius(&self) -> T;
}

/// Triangle3D Measure トレイト（4+4メソッド）
pub trait Triangle3DMeasure<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 三角形の面積を計算
    fn measure(&self) -> T;

    /// 辺ABの長さ
    fn edge_ab_length(&self) -> T;

    /// 辺BCの長さ
    fn edge_bc_length(&self) -> T;

    /// 辺CAの長さ
    fn edge_ca_length(&self) -> T;

    // ========== Phase 2 実装 ==========
    /// 周囲長を計算
    fn perimeter(&self) -> T;

    /// 点が三角形内部にあるか判定（平面投影）
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点から三角形までの距離（最短距離）
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 三角形が平面上にあるか判定
    fn is_planar(&self) -> bool;
}

/// Triangle3D Core トレイト（統合インターフェース）
pub trait Triangle3DCore<T: Scalar>:
    Triangle3DConstructor<T> + Triangle3DProperties<T> + Triangle3DMeasure<T>
{
}
