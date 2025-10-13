//! 幾何形状の中核基盤トレイト
//!
//! RedRing の Core/Extension Foundation パターンにおける中核部分
//! 全ての幾何形状が必ず実装すべき最小限の共通インターフェース
//!
//! # RedRing Core/Extension Foundation パターン
//!
//! ## 設計思想
//!
//! RedRing では幾何形状の機能を **Core（中核）** と **Extension（拡張）** に分離し、
//! 用途に応じて必要な機能のみを使用できる柔軟なアーキテクチャを採用しています。
//!
//! ### Core Foundation（必須・高速）
//! - **目的**: レンダリング・衝突判定・空間インデックスに必要な基本機能
//! - **特徴**: 軽量・高速・必須実装
//! - **含まれる機能**: 構築、アクセサ、基本計量、基本包含、基本パラメータ、境界ボックス
//!
//! ### Extension Foundation（拡張・高機能）
//! - **目的**: 高度な操作・分析・変換機能
//! - **特徴**: オプション実装・機能豊富
//! - **含まれる機能**: 高度な構築、変形、空間関係、次元変換、コレクション操作
//!
//! ## 実装パターン
//!
//! ### ファイル分離
//! ```
//! circle_2d.rs              // Core実装（120行）
//! circle_2d_extensions.rs   // Extension実装（130行）
//! ```
//!
//! ### 利用パターン
//! ```rust
//! // Core のみ使用（軽量・高速）
//! use geo_primitives::Circle2D;
//! let circle = Circle2D::new(center, radius)?;
//! let area = circle.area();
//!
//! // Extension も使用（フル機能）
//! // extensions.rs がインポートされれば自動的に拡張メソッドが利用可能
//! let unit_circle = Circle2D::unit_circle();
//! let moved_circle = circle.translate(Vector2D::new(1.0, 2.0));
//! ```
//!
//! ## トレイト階層
//!
//! ### Core Foundation トレイト
//! - `CoreFoundation<T>`: 基本属性（境界ボックス）
//! - `BasicMetrics<T>`: 基本計量（長さ、面積、体積、周長）
//! - `BasicContainment<T>`: 基本包含（点の包含判定、距離計算）
//! - `BasicParametric<T>`: 基本パラメータ（パラメータ化形状の操作）
//! - `BasicDirectional<T>`: 基本方向性（方向ベクトル、反転）
//!
//! ## メリット
//!
//! 1. **段階的実装**: 最小限から段階的に機能を追加可能
//! 2. **用途別最適化**: レンダリング用（軽量）vs 解析用（高機能）
//! 3. **保守性**: 責務分離により理解・修正が容易
//! 4. **拡張性**: 新しい Extension を後から追加可能

use crate::Scalar;

// ============================================================================
// Core Foundation - 統一基盤システム
// ============================================================================

/// 幾何形状の中核基盤トレイト
///
/// データアクセス層：基本属性の取得のみ
/// 全ての幾何形状が必ず実装する必須インターフェース
pub trait CoreFoundation<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 境界ボックスの型（BBoxに統一）
    type BBox;

    /// 境界ボックスを取得（空間インデックス・レンダリングの基盤として基本機能）
    fn bounding_box(&self) -> Self::BBox;
}

/// 基本計量特性
///
/// データ構造に直接関連する基本計算のみ
pub trait BasicMetrics<T: Scalar> {
    /// 長さを取得（線分、円弧、曲線など）
    fn length(&self) -> Option<T> {
        None
    }

    /// 面積を取得（円、楕円、多角形など）
    fn area(&self) -> Option<T> {
        None
    }

    /// 体積を取得（球、円柱、多面体など）
    fn volume(&self) -> Option<T> {
        None
    }

    /// 周長を取得（閉じた図形）
    fn perimeter(&self) -> Option<T> {
        None
    }
}

/// 基本包含判定
///
/// 点の包含関係の基本判定のみ
pub trait BasicContainment<T: Scalar>: CoreFoundation<T> {
    /// 点が図形内部に含まれるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が図形の境界上にあるかを判定（許容誤差付き）
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 点から図形への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}

/// 基本パラメータ化
///
/// パラメトリック形状の基本的なパラメータアクセス
pub trait BasicParametric<T: Scalar>: CoreFoundation<T> {
    /// パラメータ範囲を取得
    fn parameter_range(&self) -> (T, T);

    /// 指定パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定パラメータでの接線ベクトルを取得
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;
}

/// 基本方向性
///
/// 方向を持つ幾何要素の基本インターフェース
pub trait BasicDirectional<T: Scalar>: CoreFoundation<T> {
    /// 方向ベクトルの型
    type Direction;

    /// 主方向を取得
    fn direction(&self) -> Self::Direction;

    /// 方向を反転
    fn reverse_direction(&self) -> Self;
}
