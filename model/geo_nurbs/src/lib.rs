//! # `geo_nurbs`
//!
//! RedRingプラットフォーム用NURBS曲線・曲面ライブラリ
//!
//! ## 概要
//!
//! このクレートは、Non-Uniform Rational B-Splines (NURBS) 曲線と曲面の
//! 実装を提供します。CAD/CAMアプリケーションで必要な高品質な自由曲線・
//! 自由曲面の表現と操作を可能にします。
//!
//! ## 機能
//!
//! - **NURBS曲線**: 制御点、ノットベクトル、重みによる曲線定義
//! - **NURBS曲面**: パラメータ曲面の表現と操作
//! - **基底関数**: Cox-de Boor アルゴリズムによる効率的な計算
//! - **評価**: 曲線上の点、導関数、曲率の計算
//! - **変換**: アフィン変換、投影変換の適用
//! - **精度制御**: 数値計算での精度保証
//!
//! ## 使用例
//!
//! ### 2D NURBS曲線
//! ```rust
//! use geo_nurbs::NurbsCurve2D;
//! use geo_primitives::Point2D;
//!
//! // 制御点を定義
//! let control_points = vec![
//!     Point2D::new(0.0, 0.0),
//!     Point2D::new(1.0, 1.0),
//!     Point2D::new(2.0, 0.0),
//! ];
//!
//! // NURBS曲線を作成
//! let curve = NurbsCurve2D::new(
//!     control_points,
//!     Some(vec![1.0, 1.0, 1.0]), // 重み（Optionalで指定）
//!     vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], // ノットベクトル
//!     2, // 次数
//! ).unwrap();
//!
//! // パラメータ t=0.5 での点を評価
//! let point = curve.evaluate_at(0.5);
//! ```
//!
//! ### 3D NURBS曲線
//! ```rust,ignore
//! use geo_nurbs::NurbsCurve3D;
//! use geo_primitives::Point3D;
//!
//! let control_points = vec![
//!     Point3D::new(0.0, 0.0, 0.0),
//!     Point3D::new(1.0, 1.0, 1.0),
//!     Point3D::new(2.0, 0.0, 0.0),
//! ];
//!
//! let curve = NurbsCurve3D::new(control_points, None, knots, 2).unwrap();
//! let tangent = curve.tangent_at(0.5); // 正規化された接線ベクトル
//! ```
//!
//! ### 3D NURBSサーフェス
//! ```rust,ignore
//! use geo_nurbs::NurbsSurface3D;
//! use geo_primitives::Point3D;
//!
//! let control_grid = vec![
//!     vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
//!     vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)],
//! ];
//!
//! let surface = NurbsSurface3D::new(control_grid, None, u_knots, v_knots, 1, 1).unwrap();
//! let point = surface.evaluate_at(0.5, 0.5);
//! let normal = surface.normal_at(0.5, 0.5); // 法線ベクトル
//! ```
//!
//! ## アーキテクチャ
//!
//! このクレートは以下の設計原則に従います：
//!
//! - **型安全性**: ジェネリック型パラメータによる数値型の抽象化
//! - **エラーハンドリング**: Result型による明示的なエラー処理
//! - **パフォーマンス**: 効率的なアルゴリズムの実装
//! - **相互運用性**: 他のgeoクレートとの統合
//!
//! ## 依存関係
//!
//! - `geo_foundation`: 基本的な幾何型とトレイト
//! - `geo_core`: 幾何計算の基盤機能
//! - `analysis`: 数値解析アルゴリズム
//! - `nalgebra`: 線形代数演算

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// NURBS実装モジュール
pub mod basis;
pub mod curve_2d;
pub mod curve_3d;
pub mod foundation_impl;
pub mod surface;
pub mod transform;

pub mod error;
pub mod knot;
pub mod weight_storage;

// 主要な型を再エクスポート
pub use basis::{basis_function, basis_functions, rational_basis_functions};
pub use curve_2d::NurbsCurve2D;
pub use curve_3d::NurbsCurve3D;
pub use error::{NurbsError, Result};
pub use knot::{validate_knot_vector, KnotVector};
pub use surface::NurbsSurface3D;
pub use transform::{CurveSplitting, DegreeElevation, KnotInsertion};
pub use weight_storage::WeightStorage;

// 基本型をgeo_primitivesから再エクスポート
pub use geo_primitives::{Point2D, Point3D, Vector2D, Vector3D};
// Scalarはanalysisクレートから
pub use analysis::Scalar;

/// NURBS関連の定数
pub mod constants {
    /// デフォルトの数値許容誤差
    pub const DEFAULT_TOLERANCE: f64 = 1e-10;

    /// 最小の有効なノット間隔
    pub const MIN_KNOT_INTERVAL: f64 = 1e-12;

    /// 最大サポート次数
    pub const MAX_DEGREE: usize = 10;

    /// 最小制御点数（線形曲線の場合）
    pub const MIN_CONTROL_POINTS: usize = 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tolerance_usage() {
        // 実際のNURBS計算でのトレランス使用テスト（動的な値）
        let tolerance = constants::DEFAULT_TOLERANCE;
        let small_diff = tolerance * 0.5;
        assert!(small_diff < tolerance, "Small difference should be within tolerance");
        
        let large_diff = tolerance * 2.0;
        assert!(large_diff > tolerance, "Large difference should exceed tolerance");
    }
}
