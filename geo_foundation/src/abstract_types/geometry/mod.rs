//! 幾何学的要素のトレイト定義
//!
//! CAD/CAM で使用される基本的な幾何学要素のトレイト群を提供

pub mod angle;
// pub mod arc;  // 一時的にコメントアウト
pub mod bbox;
pub mod circle;
pub mod direction;
pub mod ellipse;
pub mod ellipse_arc;
pub mod infinite_line;
pub mod normalizable;
pub mod point;
pub mod vector;

// 基本トレイトをエクスポート
pub use angle::*;
// pub use arc::*;  // 一時的にコメントアウト
pub use bbox::*;
pub use circle::*;
pub use direction::*;
pub use ellipse::*;
pub use ellipse_arc::*;
pub use infinite_line::*;
pub use normalizable::*;
pub use point::*;
pub use vector::*;
