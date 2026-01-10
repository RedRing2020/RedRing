//! Vector Core Traits - Vector形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

// ============================================================================
// 1. Constructor Traits - Vector生成機能
// ============================================================================

/// Vector2D生成のためのConstructorトレイト
pub trait Vector2DConstructor<T: Scalar> {
    /// 基本コンストラクタ
    fn new(x: T, y: T) -> Self;

    /// ゼロベクトル作成
    fn zero() -> Self;

    /// X軸単位ベクトル作成
    fn unit_x() -> Self;

    /// Y軸単位ベクトル作成
    fn unit_y() -> Self;

    /// タプルから作成
    fn from_tuple(components: (T, T)) -> Self;

    /// Analysis Vector2から作成
    fn from_analysis_vector(vector: &Vector2<T>) -> Self;

    /// 配列から作成
    fn from_array(components: [T; 2]) -> Self;

    /// 極座標から作成（r, θ）
    fn from_polar(magnitude: T, angle: T) -> Self;

    /// 別のベクトルからコピー作成
    fn from_vector(other: &Self) -> Self;
}

/// Vector3D生成のためのConstructorトレイト
pub trait Vector3DConstructor<T: Scalar> {
    /// 基本コンストラクタ
    fn new(x: T, y: T, z: T) -> Self;

    /// ゼロベクトル作成
    fn zero() -> Self;

    /// X軸単位ベクトル作成
    fn unit_x() -> Self;

    /// Y軸単位ベクトル作成
    fn unit_y() -> Self;

    /// Z軸単位ベクトル作成
    fn unit_z() -> Self;

    /// タプルから作成
    fn from_tuple(components: (T, T, T)) -> Self;

    /// Analysis Vector3から作成
    fn from_analysis_vector(vector: &Vector3<T>) -> Self;

    /// 配列から作成
    fn from_array(components: [T; 3]) -> Self;

    /// 球面座標から作成（r, θ, φ）
    fn from_spherical(magnitude: T, azimuth: T, elevation: T) -> Self;

    /// 円筒座標から作成（ρ, φ, z）
    fn from_cylindrical(radial_distance: T, azimuth: T, height: T) -> Self;

    /// 別のベクトルからコピー作成
    fn from_vector(other: &Self) -> Self;
}

// ============================================================================
// 2. Properties Traits - Vector基本情報取得
// ============================================================================

/// Vector2D基本プロパティ取得トレイト
pub trait Vector2DProperties<T: Scalar> {
    /// X成分取得
    fn x(&self) -> T;

    /// Y成分取得
    fn y(&self) -> T;

    /// 成分を配列として取得
    fn components(&self) -> [T; 2];

    /// 成分をタプルとして取得
    fn to_tuple(&self) -> (T, T);

    /// Analysis Vector2へ変換
    fn to_analysis_vector(&self) -> Vector2<T>;

    /// ベクトルの長さ
    fn length(&self) -> T;

    /// ベクトルの長さの二乗（高速版）
    fn length_squared(&self) -> T;

    /// 正規化されたベクトル
    fn normalize(&self) -> Self;

    /// 安全な正規化（ゼロベクトルはNone）
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;

    /// ゼロベクトル判定
    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }

    /// 単位ベクトル判定
    fn is_unit(&self) -> bool {
        let len_sq = self.length_squared();
        (len_sq - T::ONE).abs() < T::EPSILON
    }

    /// 次元情報（Vectorは1次元）
    fn dimension(&self) -> u32 {
        1
    }
}

/// Vector3D基本プロパティ取得トレイト
pub trait Vector3DProperties<T: Scalar> {
    /// X成分取得
    fn x(&self) -> T;

    /// Y成分取得
    fn y(&self) -> T;

    /// Z成分取得
    fn z(&self) -> T;

    /// 成分を配列として取得
    fn components(&self) -> [T; 3];

    /// 成分をタプルとして取得
    fn to_tuple(&self) -> (T, T, T);

    /// Analysis Vector3へ変換
    fn to_analysis_vector(&self) -> Vector3<T>;

    /// ベクトルの長さ
    fn length(&self) -> T;

    /// ベクトルの長さの二乗（高速版）
    fn length_squared(&self) -> T;

    /// 正規化されたベクトル
    fn normalize(&self) -> Self;

    /// 安全な正規化（ゼロベクトルはNone）
    fn try_normalize(&self) -> Option<Self>
    where
        Self: Sized;

    /// ゼロベクトル判定
    fn is_zero(&self) -> bool {
        self.length_squared().is_zero()
    }

    /// 単位ベクトル判定
    fn is_unit(&self) -> bool {
        let len_sq = self.length_squared();
        (len_sq - T::ONE).abs() < T::EPSILON
    }

    /// 次元情報（Vectorは1次元）
    fn dimension(&self) -> u32 {
        1
    }
}

// ============================================================================
// 3. Transform機能は共通のAnalysisTransformトレイト使用
// （extensions/transform.rs で既に実装済み）
// ============================================================================
// Vector Transform機能は以下の既存トレイトを使用：
// - AnalysisTransform2D<T> (Vector2D用)
// - AnalysisTransform3D<T> (Vector3D用)

// ============================================================================
// 4. Measure Traits - Vector計量機能
// ============================================================================

/// Vector2D計量機能トレイト
pub trait Vector2DMeasure<T: Scalar> {
    /// 他のベクトルとの内積
    fn dot(&self, other: &Self) -> T;

    /// 他のベクトルとの2D外積（スカラー値）
    fn cross_2d(&self, other: &Self) -> T;

    /// 他のベクトルとの角度
    fn angle_to(&self, other: &Self) -> Option<T>;

    /// 他のベクトルとの距離
    fn distance_to(&self, other: &Self) -> T;

    /// 他のベクトルとの距離の二乗（高速版）
    fn distance_squared_to(&self, other: &Self) -> T;

    /// ベクトルの大きさ（magnitude、lengthと同じ）
    fn magnitude(&self) -> T;

    /// マンハッタン距離（L1ノルム）
    fn manhattan_distance(&self) -> T;

    /// 平行判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 垂直判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他のベクトルへの射影
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 面積（Vector2Dは0）
    fn area(&self) -> Option<T> {
        None
    }

    /// 長さ（ベクトルの大きさ）
    fn length(&self) -> Option<T> {
        Some(self.magnitude())
    }
}

/// Vector3D計量機能トレイト
pub trait Vector3DMeasure<T: Scalar> {
    /// 他のベクトルとの内積
    fn dot(&self, other: &Self) -> T;

    /// 他のベクトルとの3D外積（ベクトル値）
    fn cross_3d(&self, other: &Self) -> Self;

    /// 他のベクトルとの角度
    fn angle_to(&self, other: &Self) -> Option<T>;

    /// 他のベクトルとの距離
    fn distance_to(&self, other: &Self) -> T;

    /// 他のベクトルとの距離の二乗（高速版）
    fn distance_squared_to(&self, other: &Self) -> T;

    /// ベクトルの大きさ（magnitude、lengthと同じ）
    fn magnitude(&self) -> T;

    /// マンハッタン距離（L1ノルム）
    fn manhattan_distance(&self) -> T;

    /// 平行判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 垂直判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他のベクトルへの射影
    fn project_onto(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 平面への射影（法線ベクトル指定）
    fn project_onto_plane(&self, normal: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 面積（Vector3Dは0）
    fn area(&self) -> Option<T> {
        None
    }

    /// 体積（Vector3Dは0）
    fn volume(&self) -> Option<T> {
        None
    }

    /// 長さ（ベクトルの大きさ）
    fn length(&self) -> Option<T> {
        Some(self.magnitude())
    }
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// Vector2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Vector2DCore<T: Scalar>:
    Vector2DConstructor<T> + Vector2DProperties<T> + Vector2DMeasure<T>
{
}

/// Vector3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Vector3DCore<T: Scalar>:
    Vector3DConstructor<T> + Vector3DProperties<T> + Vector3DMeasure<T>
{
}

// ============================================================================
// Extension機能候補（後で所属確認）
// ============================================================================

/// Vector拡張機能の候補（現在の実装から抽出）
/// これらは後でExtensionsに移行する可能性がある
pub mod extension_candidates {
    use super::*;

    /// ベクトル間の関係演算（Extension候補）
    pub trait VectorRelations<T: Scalar> {
        /// 2つのベクトルの中間方向
        fn bisector(&self, other: &Self) -> Option<Self>
        where
            Self: Sized;

        /// ベクトルの反射（法線に対して）
        fn reflect(&self, normal: &Self) -> Self;

        /// ベクトルの屈折（スネルの法則）
        fn refract(&self, normal: &Self, eta: T) -> Option<Self>
        where
            Self: Sized;

        /// 線形補間
        fn lerp(&self, other: &Self, t: T) -> Self;

        /// 球面線形補間（単位ベクトル用）
        fn slerp(&self, other: &Self, t: T) -> Option<Self>
        where
            Self: Sized;
    }

    /// 特殊なベクトル変換（Extension候補）
    pub trait VectorSpecialTransform<T: Scalar> {
        /// 90度回転（2D）
        fn rotate_90(&self) -> Self;

        /// 180度回転
        fn rotate_180(&self) -> Self;

        /// 270度回転（2D）
        fn rotate_270(&self) -> Self;

        /// 要素ごとの変換
        fn map_components<F>(&self, f: F) -> Self
        where
            F: Fn(T) -> T;

        /// 要素ごとの積（Hadamard積）
        fn hadamard(&self, other: &Self) -> Self;
    }

    /// 複数ベクトル一括変換（Extension候補）
    /// 実装は既存のanalysis_transformモジュールで提供済み
    pub trait VectorBatchTransform<T: Scalar> {
        // 複数ベクトルの一括変換は既存のanalysis_transformモジュールで提供
        // vector_2d_transform.rs: transform_vectors_2d
        // vector_3d_transform.rs: transform_vectors_3d
    }
}
