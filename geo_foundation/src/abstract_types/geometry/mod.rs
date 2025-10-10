//! 幾何学的要素のトレイト定義
//!
//! CAD/CAM で使用される基本的な幾何学要素のトレイト群を提供

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
    CurveAnalysis3D,
    CurveType,
    DifferentialGeometry,
    Direction3DConstants,
    DirectionConstants,
    Normalizable, // commonモジュールから提供
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
