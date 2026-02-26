//! Point Core Traits - Point形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! Phase 1 + Phase 2: 3-5-4 パターン (Constructor +3, Properties +5, Measure +4)
//! 最終更新日: 2025年11月29日

use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

// ============================================================================
// 1. Constructor Traits - Point生成機能
// ============================================================================

/// Point2D生成のためのConstructorトレイト（3+3メソッド）
pub trait Point2DConstructor<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 基本コンストラクタ
    fn new(x: T, y: T) -> Self;

    /// 原点作成
    fn origin() -> Self;

    /// タプルから作成
    fn from_tuple(coords: (T, T)) -> Self;

    // ========== Phase 2 実装 ==========
    /// Analysis Vector2から作成
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;

    /// 別の点からコピー作成
    fn from_point(other: &Self) -> Self;

    /// 極座標から作成（r: 半径, theta: 角度）
    fn from_polar(r: T, theta: T) -> Self;
}

/// Point3D生成のためのConstructorトレイト（3+3メソッド）
pub trait Point3DConstructor<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 基本コンストラクタ
    fn new(x: T, y: T, z: T) -> Self;

    /// 原点作成
    fn origin() -> Self;

    /// タプルから作成
    fn from_tuple(coords: (T, T, T)) -> Self;

    // ========== Phase 2 実装 ==========
    /// Analysis Vector3から作成
    fn from_analysis_vector(vector: &Vector3<T>) -> Self;

    /// 別の点からコピー作成
    fn from_point(other: &Self) -> Self;

    /// 球面座標から作成（r: 半径, theta: 方位角, phi: 仰角）
    fn from_spherical(r: T, theta: T, phi: T) -> Self;
}

// ============================================================================
// 2. Properties Traits - Point基本情報取得
// ============================================================================

/// Point2D基本プロパティ取得トレイト（5+3メソッド）
pub trait Point2DProperties<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// X座標取得
    fn x(&self) -> T;

    /// Y座標取得
    fn y(&self) -> T;

    /// 座標を配列として取得
    fn coords(&self) -> [T; 2];

    /// 座標をタプルとして取得
    fn to_tuple(&self) -> (T, T);

    /// Analysis Vector2へ変換
    fn to_analysis_vector(&self) -> Vector2<T>;

    // ========== Phase 2 実装 ==========
    /// 位置（自分自身）
    fn position(&self) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
    }

    /// 次元情報（Pointは0次元）
    fn dimension(&self) -> u32 {
        0
    }

    /// 極座標の半径成分を取得
    fn polar_radius(&self) -> T;

    // 注: bounding_boxはExtensionFoundationで提供されます
    // 表示や交差判定等の特定用途で必要な場合のみ使用
}

/// Point3D基本プロパティ取得トレイト（5+3メソッド）
pub trait Point3DProperties<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// X座標取得
    fn x(&self) -> T;

    /// Y座標取得
    fn y(&self) -> T;

    /// Z座標取得
    fn z(&self) -> T;

    /// 座標を配列として取得
    fn coords(&self) -> [T; 3];

    /// 座標をタプルとして取得
    fn to_tuple(&self) -> (T, T, T);

    // ========== Phase 2 実装 ==========
    /// Analysis Vector3へ変換
    fn to_analysis_vector(&self) -> Vector3<T>;

    /// 位置（自分自身）
    fn position(&self) -> Self
    where
        Self: Sized + Clone,
    {
        self.clone()
    }

    /// 次元情報（Pointは0次元）
    fn dimension(&self) -> u32 {
        0
    }

    // 注: bounding_boxはExtensionFoundationで提供されます
    // 表示や交差判定等の特定用途で必要な場合のみ使用
}

// ============================================================================
// 3. Transform機能は共通のAnalysisTransformトレイト使用
// （extensions/transform.rs からcore/transform.rs に移動予定）
// ============================================================================
// Point Transform機能は以下の既存トレイトを使用：
// - AnalysisTransform2D<T> (Point2D用)
// - AnalysisTransform3D<T> (Point3D用)

// ============================================================================
// 4. Measure Traits - Point計量機能
// ============================================================================

/// Point2D計量機能トレイト（4+4メソッド）
pub trait Point2DMeasure<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 他の点までの距離
    fn distance_to(&self, other: &Self) -> T;

    /// 他の点までの距離の二乗（高速版）
    fn distance_squared_to(&self, other: &Self) -> T;

    /// 原点からの距離（ノルム）
    fn distance_from_origin(&self) -> T;

    /// 原点からの距離の二乗（高速版）
    fn norm_squared(&self) -> T;

    // ========== Phase 2 実装 ==========
    /// 2点の中点を計算
    fn midpoint(&self, other: &Self) -> Self;

    /// 別の点との線形補間
    fn lerp(&self, other: &Self, t: T) -> Self;

    /// マンハッタン距離（L1ノルム）
    fn manhattan_distance_to(&self, other: &Self) -> T;

    /// チェビシェフ距離（L∞ノルム）
    fn chebyshev_distance_to(&self, other: &Self) -> T;

    /// 面積（Pointは0）
    fn area(&self) -> Option<T> {
        None
    }

    /// 長さ（Pointは0）
    fn length(&self) -> Option<T> {
        None
    }
}

/// Point3D計量機能トレイト（4+4メソッド）
pub trait Point3DMeasure<T: Scalar> {
    // ========== Phase 1 実装 ==========
    /// 他の点までの距離
    fn distance_to(&self, other: &Self) -> T;

    /// 他の点までの距離の二乗（高速版）
    fn distance_squared_to(&self, other: &Self) -> T;

    /// 原点からの距離（ノルム）
    fn distance_from_origin(&self) -> T;

    /// 原点からの距離の二乗（高速版）
    fn norm_squared(&self) -> T;

    // ========== Phase 2 実装 ==========
    /// 2点の中点を計算
    fn midpoint(&self, other: &Self) -> Self;

    /// 別の点との線形補間
    fn lerp(&self, other: &Self, t: T) -> Self;

    /// マンハッタン距離（L1ノルム）
    fn manhattan_distance_to(&self, other: &Self) -> T;

    /// チェビシェフ距離（L∞ノルム）
    fn chebyshev_distance_to(&self, other: &Self) -> T;

    /// 面積（Pointは0）
    fn area(&self) -> Option<T> {
        None
    }

    /// 体積（Pointは0）
    fn volume(&self) -> Option<T> {
        None
    }

    /// 長さ（Pointは0）
    fn length(&self) -> Option<T> {
        None
    }
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// Point2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Point2DCore<T: Scalar>:
    Point2DConstructor<T> + Point2DProperties<T> + Point2DMeasure<T>
{
}

/// Point3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Point3DCore<T: Scalar>:
    Point3DConstructor<T> + Point3DProperties<T> + Point3DMeasure<T>
{
}

// ============================================================================
// Extension機能候補（後で所属確認）
// ============================================================================

/// Point拡張機能の候補（現在の実装から抽出）
/// これらは後でExtensionsに移行する可能性がある
pub mod extension_candidates {
    use super::*;

    /// 境界判定機能（Extension候補）
    pub trait PointBoundaryTest<T: Scalar> {
        /// 点が境界上（許容誤差内）にあるかを判定
        fn on_boundary(&self, point: &Self, tolerance: T) -> bool;

        /// 点が自分自身と一致するかを判定
        fn contains_point(&self, point: &Self) -> bool;
    }

    /// 複数点一括変換（Extension候補）
    /// 実装は既存のanalysis_transformモジュールで提供済み
    pub trait PointBatchTransform<T: Scalar> {
        // 複数点の一括変換は既存のanalysis_transformモジュールで提供
        // point_2d_transform.rs: transform_points_2d
        // point_3d_transform.rs: transform_points_3d (未実装の場合)
    }
}
