//! NURBS（Non-Uniform Rational B-Splines）関連のトレイト定義
//!
//! このモジュールは、NURBS曲線とサーフェスの抽象化層を提供し、
//! 型安全性とコードの再利用性を実現します。

use crate::Scalar;

/// NURBS曲線のボックス化された動的トレイトオブジェクト型エイリアス
pub type BoxedNurbsCurve<T, P, V> = Box<dyn NurbsCurve<T, Point = P, Vector = V>>;

/// NURBS曲線の分割結果型エイリアス
pub type CurveSplitResult<T, P, V> =
    Result<(BoxedNurbsCurve<T, P, V>, BoxedNurbsCurve<T, P, V>), String>;

/// NURBSサーフェスのボックス化された動的トレイトオブジェクト型エイリアス
pub type BoxedNurbsSurface<T, P, V> = Box<dyn NurbsSurface<T, Point = P, Vector = V>>;

/// NURBSサーフェスの分割結果型エイリアス
pub type SurfaceSplitResult<T, P, V> =
    Result<(BoxedNurbsSurface<T, P, V>, BoxedNurbsSurface<T, P, V>), String>;

/// ノットベクトルの抽象化
pub trait KnotVector<T: Scalar> {
    /// ノットベクトルの長さを取得
    fn len(&self) -> usize;

    /// ノットベクトルが空かどうか確認
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// インデックスによるノット値の取得
    fn knot_at(&self, index: usize) -> T;

    /// パラメータ定義域を取得
    fn parameter_domain(&self, degree: usize) -> (T, T);

    /// ノットベクトルの有効性を検証
    fn validate(&self, degree: usize, control_point_count: usize) -> Result<(), String>;

    /// ノットスパンを見つける
    fn find_span(&self, parameter: T, degree: usize) -> usize;
}

/// NURBS曲線の共通インターフェース
pub trait NurbsCurve<T: Scalar> {
    /// 点の型（2D/3D）
    type Point;

    /// ベクトルの型（2D/3D）
    type Vector;

    /// 曲線の次数を取得
    fn degree(&self) -> usize;

    /// 制御点の数を取得
    fn control_point_count(&self) -> usize;

    /// パラメータ定義域を取得
    fn parameter_domain(&self) -> (T, T);

    /// 指定パラメータでの曲線上の点を評価
    fn evaluate_at(&self, parameter: T) -> Self::Point;

    /// 指定パラメータでの1次導関数を計算
    fn derivative_at(&self, parameter: T) -> Self::Vector;

    /// 指定パラメータでの接線ベクトルを計算
    fn tangent_at(&self, parameter: T) -> Self::Vector {
        self.derivative_at(parameter)
    }

    /// 曲線が有理（重み付き）かどうか確認
    fn is_rational(&self) -> bool;

    /// 曲線が閉じているかどうか確認
    fn is_closed(&self, tolerance: T) -> bool;

    /// 曲線の長さを近似計算
    fn approximate_length(&self, subdivisions: usize) -> T;
}

/// NURBS曲線の高度な操作
pub trait NurbsCurveOperations<T: Scalar>: NurbsCurve<T> {
    /// ノット挿入
    fn insert_knot(&mut self, parameter: T, multiplicity: usize) -> Result<(), String>;

    /// 次数昇格
    fn elevate_degree(&mut self, target_degree: usize) -> Result<(), String>;

    /// 曲線の分割
    fn split_at(&self, parameter: T) -> CurveSplitResult<T, Self::Point, Self::Vector>;

    /// 制御点の追加
    fn add_control_point(&mut self, point: Self::Point, weight: Option<T>) -> Result<(), String>;

    /// 制御点の削除
    fn remove_control_point(&mut self, index: usize) -> Result<(), String>;
}

/// NURBSサーフェスの共通インターフェース
pub trait NurbsSurface<T: Scalar> {
    /// 点の型（通常は3D）
    type Point;

    /// ベクトルの型（通常は3D）
    type Vector;

    /// u方向の次数を取得
    fn u_degree(&self) -> usize;

    /// v方向の次数を取得
    fn v_degree(&self) -> usize;

    /// 制御点グリッドのサイズを取得 (u_count, v_count)
    fn grid_size(&self) -> (usize, usize);

    /// パラメータ定義域を取得 ((u_min, u_max), (v_min, v_max))
    fn parameter_domain(&self) -> ((T, T), (T, T));

    /// 指定パラメータでのサーフェス上の点を評価
    fn evaluate_at(&self, u: T, v: T) -> Self::Point;

    /// u方向の偏導関数を計算
    fn u_derivative_at(&self, u: T, v: T) -> Self::Vector;

    /// v方向の偏導関数を計算
    fn v_derivative_at(&self, u: T, v: T) -> Self::Vector;

    /// 指定点での法線ベクトルを計算
    fn normal_at(&self, u: T, v: T) -> Self::Vector;

    /// サーフェスが有理（重み付き）かどうか確認
    fn is_rational(&self) -> bool;

    /// サーフェスが閉じているかどうか確認（u方向）
    fn is_u_closed(&self, tolerance: T) -> bool;

    /// サーフェスが閉じているかどうか確認（v方向）
    fn is_v_closed(&self, tolerance: T) -> bool;

    /// サーフェスの面積を近似計算
    fn approximate_area(&self, u_subdivisions: usize, v_subdivisions: usize) -> T;
}

/// NURBSサーフェスの高度な操作
pub trait NurbsSurfaceOperations<T: Scalar>: NurbsSurface<T> {
    /// u方向のノット挿入
    fn insert_u_knot(&mut self, parameter: T, multiplicity: usize) -> Result<(), String>;

    /// v方向のノット挿入
    fn insert_v_knot(&mut self, parameter: T, multiplicity: usize) -> Result<(), String>;

    /// u方向の次数昇格
    fn elevate_u_degree(&mut self, target_degree: usize) -> Result<(), String>;

    /// v方向の次数昇格
    fn elevate_v_degree(&mut self, target_degree: usize) -> Result<(), String>;

    /// u方向でのサーフェス分割
    fn split_u_at(&self, parameter: T) -> SurfaceSplitResult<T, Self::Point, Self::Vector>;

    /// v方向でのサーフェス分割
    fn split_v_at(&self, parameter: T) -> SurfaceSplitResult<T, Self::Point, Self::Vector>;
}

/// 重み管理の抽象化
pub trait WeightedGeometry<T: Scalar> {
    /// 重みを取得（インデックス指定）
    fn weight_at(&self, index: usize) -> T;

    /// 重みを設定（インデックス指定）
    fn set_weight_at(&mut self, index: usize, weight: T) -> Result<(), String>;

    /// 全ての重みが均等（= 1.0）かどうか確認
    fn is_uniform_weight(&self) -> bool;

    /// 非有理形式に変換（全重み = 1.0）
    fn make_non_rational(&mut self);

    /// 有理形式に変換（個別重み設定）
    fn make_rational(&mut self, weights: Vec<T>) -> Result<(), String>;
}

/// NURBS基底関数の抽象化
pub trait BasisFunction<T: Scalar> {
    /// B-spline基底関数の計算
    fn compute_basis_functions(
        knots: &dyn KnotVector<T>,
        span: usize,
        parameter: T,
        degree: usize,
    ) -> Vec<T>;

    /// B-spline基底関数の導関数計算
    fn compute_basis_derivatives(
        knots: &dyn KnotVector<T>,
        span: usize,
        parameter: T,
        degree: usize,
        derivative_order: usize,
    ) -> Vec<Vec<T>>;

    /// 有理基底関数（NURBS）の計算
    fn compute_rational_basis_functions(
        knots: &dyn KnotVector<T>,
        span: usize,
        parameter: T,
        degree: usize,
        weights: &[T],
    ) -> Vec<T>;
}

/// パラメータ空間での操作
pub trait ParametricGeometry<T: Scalar> {
    /// パラメータの正規化（0.0-1.0の範囲に変換）
    fn normalize_parameter(&self, parameter: T) -> T;

    /// 正規化されたパラメータを実際の範囲に変換
    fn denormalize_parameter(&self, normalized_parameter: T) -> T;

    /// パラメータがドメイン内にあるかチェック
    fn is_parameter_valid(&self, parameter: T) -> bool;

    /// パラメータをドメイン内にクランプ
    fn clamp_parameter(&self, parameter: T) -> T;
}

/// 双パラメータ（サーフェス）での操作
pub trait BiParametricGeometry<T: Scalar> {
    /// uパラメータの正規化
    fn normalize_u_parameter(&self, u: T) -> T;

    /// vパラメータの正規化
    fn normalize_v_parameter(&self, v: T) -> T;

    /// 正規化されたuパラメータを実際の範囲に変換
    fn denormalize_u_parameter(&self, normalized_u: T) -> T;

    /// 正規化されたvパラメータを実際の範囲に変換
    fn denormalize_v_parameter(&self, normalized_v: T) -> T;

    /// パラメータペアがドメイン内にあるかチェック
    fn are_parameters_valid(&self, u: T, v: T) -> bool;

    /// パラメータペアをドメイン内にクランプ
    fn clamp_parameters(&self, u: T, v: T) -> (T, T);
}
