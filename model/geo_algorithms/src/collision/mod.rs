//! 衝突判定・交差判定アルゴリズム
//!
//! 異なるクレート間の形状の衝突判定を実装します。
//! - NURBS × Primitives
//! - NURBS × NURBS (将来実装)

pub mod nurbs_3d;
pub mod pair_base;
pub mod primitive_2d;
pub mod primitive_3d;

pub use nurbs_3d::*;
pub use primitive_2d::*;
pub use primitive_3d::*;
