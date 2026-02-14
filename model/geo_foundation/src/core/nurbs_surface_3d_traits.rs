//! NurbsSurface3D Core Traits - NURBSサーフェスの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1 実装（最小限のメソッドのみ）
//! - Constructor: 3メソッド（new, from_control_points, unit_plane）
//! - Properties: 7メソッド（u_degree, v_degree, u_count, v_count, u_knots, v_knots, is_rational）
//! - Measure: 4メソッド（point_at_uv, normal_at, surface_area, tangent_vectors_at）
//!
//! 作成日: 2025年11月29日

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - NurbsSurface3D生成機能（Phase 1: 最小限）
// ============================================================================

/// NurbsSurface3D生成のためのConstructorトレイト
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
    ///
    /// # Errors
    /// * 制御点と重みのサイズが一致しない場合
    /// * ノットベクトルが無効な場合
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
    ///
    /// 自動的にクランプされた均一ノットベクトルを生成
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

// ============================================================================
// 2. Properties Traits - NurbsSurface3D基本情報取得（Phase 1: 最小限）
// ============================================================================

/// NurbsSurface3D基本プロパティ取得トレイト
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

    /// 全制御点座標のflattened配列への参照を取得
    ///
    /// # 戻り値
    /// [x00, y00, z00, x01, y01, z01, ...] の形式のスライス参照
    /// （u方向優先でflatten）
    ///
    /// # 用途
    /// GPU評価データ変換など、全制御点座標を一括で取得する場合に使用
    fn coordinates(&self) -> &[T];

    /// 全重みのflattened配列への参照を取得
    ///
    /// # 戻り値
    /// 有理サーフェスの場合は`Some(&[T])`、非有理サーフェスの場合は`None`
    /// （u方向優先でflatten）
    fn weights(&self) -> Option<&[T]>;
}

// ============================================================================
// 3. Measure Traits - NurbsSurface3D測定・評価機能（Phase 1: 最小限）
// ============================================================================

/// NurbsSurface3D測定・評価機能トレイト
pub trait NurbsSurface3DMeasure<T: Scalar> {
    /// パラメータ座標(u, v)でのサーフェス上の点を計算
    ///
    /// # Arguments
    /// * `u` - u方向パラメータ値（通常 [0, 1] の範囲）
    /// * `v` - v方向パラメータ値（通常 [0, 1] の範囲）
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T);

    /// パラメータ座標(u, v)での法線ベクトルを計算
    ///
    /// # Arguments
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    fn normal_at(&self, u: T, v: T) -> (T, T, T);

    /// サーフェスの表面積を計算
    ///
    /// 数値積分により近似計算
    fn surface_area(&self) -> T;

    /// パラメータ座標(u, v)での接線ベクトル(du, dv)を計算
    ///
    /// # Returns
    /// (du: u方向接線, dv: v方向接線)
    fn tangent_vectors_at(&self, u: T, v: T) -> ((T, T, T), (T, T, T));
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// NurbsSurface3DのCore機能を統合するトレイト
pub trait NurbsSurface3DCore<T: Scalar>:
    NurbsSurface3DConstructor<T> + NurbsSurface3DProperties<T> + NurbsSurface3DMeasure<T>
{
}
