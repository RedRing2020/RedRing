//! 幾何学的要素のトレイト定義
//!
//! CAD/CAM で使用される基本的な幾何学要素のトレイト群を提供

pub mod bbox;
pub mod direction;
pub mod normalizable;
pub mod point;
pub mod vector;

// 基本トレイトをエクスポート
pub use bbox::*;
pub use direction::*;
pub use normalizable::*;
pub use point::*;
pub use vector::*;
