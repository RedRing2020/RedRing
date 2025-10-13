//! 幾何学的要素のトレイト定義
//!
//! CAD/CAM で使用される基本的な幾何学要素のトレイト群を提供
//!
//! # 新しい設計方針: 基本・拡張境界の明確化
//!
//! ## 基本トレイト (Core Foundation)
//! データ構造・幾何構造に応じた基本の解析のみ
//!
//! ### データアクセス層
//! - `CoreFoundation`: 基本属性アクセス（境界ボックスを含む）
//! - `PointCore`, `VectorCore`: 基本的な座標・成分アクセス
//! - `CircleCore`, `LineSegmentCore`: 各形状の基本属性アクセス
//!
//! ### 基本幾何解析層
//! - `BasicMetrics`: データ構造に直接関連する計算（長さ、面積、体積）
//! - `BasicContainment`: 基本的な包含判定
//! - `BasicParametric`: パラメトリック形状の基本操作
//!
//! ## 拡張トレイト (Extension Foundation)
//! 交差・衝突・変換・射影等の高度な操作は `extension_foundation` モジュールで定義
//!
//! # エラー処理の設計方針
//!
//! RedRing プロジェクトでは、各幾何形状専用のエラー型を使用する設計を採用しています：
//!
//! ## 専用エラー型の例
//!
//! - `EllipseError` - 楕円作成・操作時のエラー
//!   - `InvalidAxisLength` - 軸長が無効
//!   - `InvalidAxisOrder` - 軸の順序が無効
//!
//! - `VectorNormalizationError` - ベクトル正規化時のエラー
//!   - `ZeroLength` - ゼロ長ベクトル
//!   - `NumericalInstability` - 数値不安定
//!
//! - `CircleError` - 円作成・操作時のエラー
//!   - `InvalidRadius` - 半径が無効
//!   - `CollinearPoints` - 3点が一直線上
//!
//! ## 新しいエラー型の追加ガイドライン
//!
//! 1. **専用性**: 各幾何形状で専用のエラー型を定義
//! 2. **具体性**: 汎用的な `GeometryError` ではなく、具体的なエラー情報を提供
//! 3. **場所**: エラー型は対象の幾何形状と同じモジュール内で定義
//! 4. **命名**: `<GeometryType>Error` のパターンを使用
//! 5. **トレイト実装**: `std::fmt::Display` と `std::error::Error` を実装
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use geo_primitives::geometry2d::{Ellipse, EllipseError, Point};
//!
//! let center = Point::new(0.0, 0.0);
//! let angle = 0.0;
//! match Ellipse::new(center, -1.0, 2.0, angle) {
//!     Ok(ellipse) => { /* 成功処理 */ },
//!     Err(EllipseError::InvalidAxisLength) => { /* エラー処理 */ },
//!     Err(e) => { /* その他のエラー */ },
//! }
//! ```

// 新しいトレイト設計（Core/Extension Foundation ベース）
pub mod core_foundation; // 中核基盤トレイト（旧 foundation）
pub mod extension_foundation; // 拡張基盤トレイト（旧 primitive）

// 統一交点計算トレイト
pub use intersection::{
    BasicIntersection, IntersectionHelpers, MultipleIntersection, SelfIntersection,
};

// 新実装専用モジュール（これから追加）
// pub mod new_point;        // 新しい点の実装
// pub mod new_vector;       // 新しいベクトルの実装
// pub mod new_circle;       // 新しい円の実装

// 早まった実装・旧トレイト（一時除外）
// pub mod helpers;          // 早まった実装 - 削除候補
// pub mod basic_point;      // 旧実装 - 一時除外
// pub mod basic_vector;     // 旧実装 - 一時除外
// pub mod basic_circle;     // 旧実装 - 一時除外
// pub mod basic_shapes;     // 旧実装 - 一時除外
// pub mod arc;              // 旧実装 - 一時除外
pub mod bbox;
pub mod circle;
pub mod classification; // 幾何プリミティブの分類システム
pub mod common; // 共通インターフェイスとヘルパー
pub mod direction;
pub mod ellipse;
pub mod ellipse_arc;
pub mod infinite_line;
pub mod intersection; // 交点計算トレイト
pub mod line;
pub mod point;
pub mod ray;
pub mod sphere;
pub mod utils; // 幾何計算ユーティリティ
pub mod vector;

// 基本トレイトをエクスポート
pub use core_foundation::*; // 中核基盤トレイト
pub use extension_foundation::*; // 拡張基盤トレイト

// 新実装の準備（将来追加予定）
// pub use new_point::{...};
// pub use new_vector::{...};
// pub use new_circle::{...};

// geo_foundation固有のヘルパー関数（新実装のため一時的に無効化）
/*
pub use helpers::{
    normalize_parameter, parameter_in_range, lerp, inverse_lerp,
    angle_to_parameter, parameter_to_angle
};
*/

// 古いトレイトのエクスポート（新実装のため一時的に無効化）
// pub use arc::*; // 有効化
// pub use bbox::*;
// pub use circle::*;
// pub use classification::*; // プリミティブ分類システム
// pub use common::*; // Direction3DConstants の重複を避けるため個別インポートに変更
/*
pub use common::{
    AnalyticalCurve,
    CollectionDistanceCalculation,
    ConditionalNormalizable,
    CurveAnalysis3D,
    CurveType,
    DifferentialGeometry,
    Direction3DConstants,
    DirectionConstants,
    // 距離計算トレイト（distance_operations モジュールから）
    DistanceCalculation,
    DistanceWithClosestPoint,
    // 正規化トレイト（normalization_operations モジュールから）
    Normalizable,
    NormalizationError,
};
*/
// pub use direction::*;
// pub use ellipse::*; // 有効化
// pub use ellipse_arc::*; // 有効化
// pub use infinite_line::*;
// pub use line::*;
// pub use point::*;
// pub use primitive::*; // 幾何プリミティブの共通トレイト
// pub use ray::*;
// pub use sphere::*;
// pub use utils::*; // ユーティリティ関数
// pub use vector::*;
