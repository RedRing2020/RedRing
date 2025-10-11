//! 幾何学的要素のトレイト定義
//!
//! CAD/CAM で使用される基本的な幾何学要素のトレイト群を提供
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
//! - `NormalizationError` - ベクトル正規化時のエラー
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

pub mod arc; // 有効化
pub mod bbox;
pub mod circle;
pub mod classification; // 幾何プリミティブの分類システム
pub mod common; // 共通インターフェイスとヘルパー
pub mod direction;
pub mod ellipse;
pub mod ellipse_arc;
pub mod infinite_line;
pub mod line;
pub mod point;
pub mod primitive; // 幾何プリミティブの共通トレイト
pub mod ray;
pub mod sphere;
pub mod utils; // 幾何計算ユーティリティ
pub mod vector;

// 基本トレイトをエクスポート
pub use arc::*; // 有効化
pub use bbox::*;
pub use circle::*;
pub use classification::*; // プリミティブ分類システム
                           // pub use common::*; // Direction3DConstants の重複を避けるため個別インポートに変更
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
pub use direction::*;
pub use ellipse::*; // 有効化
pub use ellipse_arc::*; // 有効化
pub use infinite_line::*;
pub use line::*;
pub use point::*;
pub use primitive::*; // 幾何プリミティブの共通トレイト
pub use ray::*;
pub use sphere::*;
pub use utils::*; // ユーティリティ関数
pub use vector::*;
