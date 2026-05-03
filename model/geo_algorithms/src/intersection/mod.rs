//! 交点計算アルゴリズム
//!
//! 各形状組み合わせの交点計算エントリポイントを提供する。
//! `common` は共通ヘルパーを提供し、`pair_base` は共通の基礎計算を担い、
//! `primitive_*` と `nurbs_3d` は形状種別ごとの公開 API を提供する。

pub(crate) mod common;
pub mod nurbs_3d;
pub mod pair_base;
pub mod primitive_2d;
pub mod primitive_3d;

pub use nurbs_3d::*;
pub use primitive_2d::*;
pub use primitive_3d::*;
