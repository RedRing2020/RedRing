//! 交点計算アルゴリズム
//!
//! 各形状組み合わせの交点計算エントリポイントを提供する。
//! `common` と `pair_base` はクレート内部の基礎計算を担い、
//! `primitive_*` と `nurbs_3d` が形状種別ごとの公開 API（正本）を提供する。

pub(crate) mod common;
pub mod nurbs_3d;
pub(crate) mod pair_base;
pub mod primitive_2d;
pub mod primitive_3d;

pub use nurbs_3d::*;
pub use primitive_2d::*;
pub use primitive_3d::*;
