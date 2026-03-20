//! 交点計算アルゴリズム
//!
//! Phase C では、`geo_primitives` 側の交点計算 trait実装を
//! 段階的にこのモジュール配下へ集約する。
//!
//! 現時点では受け皿のみを用意し、実装移設は後続コミットで行う。

pub mod pair_base;
pub mod primitive_2d;
pub mod primitive_3d;

pub use primitive_2d::*;
pub use primitive_3d::*;
